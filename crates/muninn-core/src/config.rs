//! Scan configuration.
//!
//! Maps directly to the Muninn Input Specification. Fields are grouped by
//! who provides them: the main GUI surfaces scan roots + output + consent,
//! the advanced panel exposes everything else, and baked-in defaults live
//! as `Default` impls.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level scan configuration. Constructed by the GUI/CLI before being
/// handed to [`crate::Scanner`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    // ── Main GUI inputs ────────────────────────────────────────────────
    /// One or more folder paths to scan. The only mandatory input.
    pub scan_roots: Vec<PathBuf>,

    /// Where to write the report JSON. Defaults to `~/Desktop`.
    pub output_dir: PathBuf,

    /// Include machine hostname and OS in the report.
    pub include_hostname: bool,

    /// If false, individual filenames are hashed (directory structure preserved).
    pub include_full_paths: bool,

    // ── Advanced panel ─────────────────────────────────────────────────
    /// Glob patterns for paths/extensions to skip. Merged with defaults.
    pub exclusion_patterns: Vec<String>,

    /// Skip files above this size in bytes. Default: 2 GB.
    pub max_file_size_bytes: u64,

    /// How many pages to sample per document for text analysis. Default: 5.
    pub text_extraction_depth: usize,

    /// Custom regex patterns for cross-reference extraction.
    /// Merged with the baked-in defaults (ISO, ANSI, ASTM, CFR, etc.).
    pub custom_xref_patterns: Vec<String>,

    /// Explicit directory-to-department mappings.
    /// Key: glob-style directory pattern, Value: department name.
    pub department_overrides: HashMap<String, String>,

    /// Number of parallel file-processing threads. Default: num_cpus - 1.
    pub concurrency: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        let num_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        Self {
            scan_roots: Vec::new(),
            output_dir: dirs_default_desktop(),
            include_hostname: true,
            include_full_paths: true,
            exclusion_patterns: default_exclusion_patterns(),
            max_file_size_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            text_extraction_depth: 5,
            custom_xref_patterns: Vec::new(),
            department_overrides: HashMap::new(),
            concurrency: num_cpus.saturating_sub(1).max(1),
        }
    }
}

/// Baked-in default exclusion patterns from the spec.
fn default_exclusion_patterns() -> Vec<String> {
    [
        // OS artifacts
        "Thumbs.db",
        ".DS_Store",
        "desktop.ini",
        "__MACOSX/**",
        // Temp / lock files
        "*.tmp",
        "*.bak",
        "~$*",
        "*.swp",
        "*.lock",
        // Version control
        ".git/**",
        ".svn/**",
        ".hg/**",
        // Dependencies
        "node_modules/**",
        "vendor/**",
        "target/**",
        "__pycache__/**",
        ".venv/**",
        "venv/**",
        // Build output
        "*.o",
        "*.obj",
        "*.class",
        "*.pyc",
        // Disk images
        "*.iso",
        "*.img",
        "*.vmdk",
        "*.vhd",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Best-effort default Desktop path.
fn dirs_default_desktop() -> PathBuf {
    // TODO: Use `dirs` crate for cross-platform Desktop detection.
    // For now, fall back to current directory.
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
