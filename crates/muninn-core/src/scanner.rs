//! Scanner — the top-level pipeline that ties everything together.
//!
//! ## Pipeline
//!
//! 1. Validate scan roots exist and are accessible.
//! 2. Compile exclusion globs and cross-reference patterns.
//! 3. Walk each scan root with `walkdir`, filtering excluded paths/sizes.
//! 4. For each file (parallel via `rayon`):
//!    a. Extract filesystem metadata (common to all depths).
//!    b. Classify by extension → inspection depth + parser kind.
//!    c. Dispatch to format-specific parser (if deep or medium).
//!    d. Run analysis passes: version detection, department inference,
//!    cross-reference extraction, language detection, jargon extraction.
//! 5. Aggregate per-document results into a corpus summary.
//! 6. Assemble and serialize the final `ScanReport`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::Utc;
use globset::{Glob, GlobSetBuilder};
use rayon::prelude::*;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::analysis;
use crate::classify::{self, InspectionDepth, ParserKind};
use crate::config::ScanConfig;
use crate::error::MuninnError;
use crate::output::{ScanMetadata, ScanReport};
use crate::parsers::common;
use crate::parsers::FormatParser;

/// The scanner. Construct with a [`ScanConfig`], call [`Scanner::run`].
pub struct Scanner {
    config: ScanConfig,
}

impl Scanner {
    #[must_use]
    pub fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    /// Execute the scan and produce a report.
    ///
    /// # Errors
    ///
    /// Returns [`MuninnError`] if a scan root is missing, a glob pattern is
    /// invalid, or a filesystem walk fails at the root.
    ///
    /// # Panics
    ///
    /// Panics only if building a fallback `rayon::ThreadPool` fails, which
    /// would indicate a broken runtime environment.
    pub fn run(&self) -> crate::Result<ScanReport> {
        let start = Instant::now();
        info!(
            "Starting scan with {} root(s)",
            self.config.scan_roots.len()
        );

        // ── 1. Validate scan roots ─────────────────────────────────────
        for root in &self.config.scan_roots {
            if !root.exists() {
                return Err(MuninnError::ScanRootNotFound { path: root.clone() });
            }
        }

        // ── 2. Compile exclusion globs ─────────────────────────────────
        let exclusion_set = {
            let mut builder = GlobSetBuilder::new();
            for pattern in &self.config.exclusion_patterns {
                builder.add(Glob::new(pattern)?);
            }
            builder.build()?
        };

        // ── 3. Collect file paths ──────────────────────────────────────
        let mut file_paths: Vec<(PathBuf, PathBuf)> = Vec::new(); // (absolute, scan_root)

        for root in &self.config.scan_roots {
            let walker = WalkDir::new(root).follow_links(false);
            for entry in walker {
                let entry = entry?;
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path().to_path_buf();

                // Check exclusion
                let relative = path.strip_prefix(root).unwrap_or(&path);
                if exclusion_set.is_match(relative) {
                    debug!("Excluded: {}", relative.display());
                    continue;
                }

                // Check file size
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() > self.config.max_file_size_bytes {
                        debug!("Skipped (too large): {}", path.display());
                        continue;
                    }
                }

                file_paths.push((path, root.clone()));
            }
        }

        info!("Found {} files to process", file_paths.len());

        // ── 4. Configure thread pool ───────────────────────────────────
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.config.concurrency)
            .build()
            .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

        // ── 5. Process files in parallel ───────────────────────────────
        let text_depth = self.config.text_extraction_depth;
        let department_overrides = &self.config.department_overrides;

        let documents: Vec<_> = pool.install(|| {
            file_paths
                .par_iter()
                .filter_map(|(path, scan_root)| {
                    process_file(path, scan_root, text_depth, department_overrides)
                })
                .collect()
        });

        // ── 6. Aggregate summary ───────────────────────────────────────
        let corpus_summary = analysis::summary::compute_summary(&documents);

        // ── 7. Build scan metadata ─────────────────────────────────────
        let scan_metadata = ScanMetadata {
            scanner_version: env!("CARGO_PKG_VERSION").to_string(),
            scan_timestamp: Utc::now(),
            scan_roots: self
                .config
                .scan_roots
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
            scan_duration_seconds: start.elapsed().as_secs_f64(),
            hostname: if self.config.include_hostname {
                get_hostname()
            } else {
                None
            },
            os: if self.config.include_hostname {
                Some(get_os())
            } else {
                None
            },
        };

        Ok(ScanReport {
            scan_metadata,
            corpus_summary,
            documents,
        })
    }
}

/// Process a single file through the full pipeline.
fn process_file(
    path: &Path,
    scan_root: &Path,
    text_depth: usize,
    department_overrides: &std::collections::HashMap<String, String>,
) -> Option<crate::output::DocumentMetadata> {
    // 4a. Filesystem metadata
    let mut doc = match common::extract_filesystem_metadata(path) {
        Ok(d) => d,
        Err(e) => {
            warn!("Failed to read metadata for {}: {}", path.display(), e);
            return None;
        }
    };

    // Fill in relative path and scan root
    let relative = path.strip_prefix(scan_root).unwrap_or(path);
    doc.relative_path = relative.display().to_string();
    doc.scan_root = scan_root.display().to_string();

    // 4b. Classify
    let classification = classify::classify_by_extension(path);

    // 4c. Dispatch to parser
    if classification.depth != InspectionDepth::Shallow {
        let parser = get_parser(classification.parser);
        if let Some(p) = parser {
            let errors = p.parse(path, &mut doc, text_depth);
            if !errors.is_empty() {
                doc.errors = Some(errors);
            }
        }
    }

    // 4d. Analysis passes
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let version = analysis::version::detect_version(stem);
    if version.filename_version.is_some() {
        doc.version = Some(crate::output::VersionInfo {
            filename_version: version.filename_version,
            version_family: version.version_family,
        });
    }

    doc.inferred_department =
        analysis::department::infer_department(&doc.relative_path, scan_root, department_overrides);

    Some(doc)
}

/// Resolve a [`ParserKind`] to a concrete [`FormatParser`] implementation.
fn get_parser(kind: ParserKind) -> Option<Box<dyn FormatParser>> {
    match kind {
        #[cfg(feature = "pdf")]
        ParserKind::Pdf => Some(Box::new(crate::parsers::pdf::PdfParser)),

        #[cfg(feature = "ooxml")]
        ParserKind::Ooxml => Some(Box::new(crate::parsers::ooxml::OoxmlParser)),

        #[cfg(feature = "ole")]
        ParserKind::Ole => Some(Box::new(crate::parsers::ole::OleParser)),

        ParserKind::Rtf => Some(Box::new(crate::parsers::rtf::RtfParser)),
        ParserKind::OpenDocument => {
            Some(Box::new(crate::parsers::opendocument::OpenDocumentParser))
        }
        ParserKind::PlainText => Some(Box::new(crate::parsers::plain_text::PlainTextParser)),
        ParserKind::Html => Some(Box::new(crate::parsers::html::HtmlParser)),

        #[cfg(feature = "email")]
        ParserKind::Eml => Some(Box::new(crate::parsers::email::EmlParser)),

        #[cfg(feature = "image-meta")]
        ParserKind::Image => Some(Box::new(crate::parsers::image::ImageParser)),

        ParserKind::ArchiveContainer => Some(Box::new(crate::parsers::archive::ArchiveParser)),

        // Formats without parsers yet
        ParserKind::CadDxf
        | ParserKind::CadStep
        | ParserKind::CadIfc
        | ParserKind::Ebook
        | ParserKind::Database => None,

        ParserKind::None => None,

        // Catch disabled features
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

fn get_hostname() -> Option<String> {
    hostname::get().ok().and_then(|h| h.into_string().ok())
}

fn get_os() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}
