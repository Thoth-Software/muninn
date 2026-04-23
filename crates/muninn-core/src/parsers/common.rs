//! Common extraction utilities shared across parsers.
//!
//! Filesystem metadata (size, dates, MIME type) is extracted here before
//! any format-specific parser runs.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::output::DocumentMetadata;

/// Populate the filesystem-level fields of a [`DocumentMetadata`].
///
/// This runs for *every* file regardless of inspection depth.
///
/// # Errors
///
/// Returns an error if `fs::metadata` fails on `path` (permission denied,
/// path does not exist, etc.).
pub fn extract_filesystem_metadata(path: &Path) -> Result<DocumentMetadata> {
    let metadata = fs::metadata(path)?;
    let file_size_bytes = metadata.len();

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mime_type = detect_mime(path);

    let fs_created =
        filetime::FileTime::from_creation_time(&metadata).and_then(filetime_to_datetime);
    let fs_modified_dt =
        filetime_to_datetime(filetime::FileTime::from_last_modification_time(&metadata));

    Ok(DocumentMetadata {
        relative_path: String::new(), // filled in by scanner after hashing decision
        scan_root: String::new(),     // filled in by scanner
        file_size_bytes,
        extension,
        mime_type,
        dates: crate::output::DateInfo {
            filesystem_created: fs_created,
            filesystem_modified: fs_modified_dt,
            document_created: None,
            document_modified: None,
        },
        authorship: None,
        version: None,
        encoding: None,
        encoding_confidence: None,
        language: None,
        language_confidence: None,
        structure: None,
        pdf_specific: None,
        inferred_department: None,
        cross_references: None,
        domain_terms: None,
        errors: None,
    })
}

fn filetime_to_datetime(ft: filetime::FileTime) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(ft.unix_seconds(), ft.nanoseconds())
}

/// Two-pass MIME detection: magic bytes first (via `infer`), extension fallback.
fn detect_mime(path: &Path) -> Option<String> {
    // Try magic bytes first — catches mismatched extensions.
    if let Ok(buf) = fs::read(path).map(|b| b.into_iter().take(8192).collect::<Vec<_>>()) {
        if let Some(kind) = infer::get(&buf) {
            return Some(kind.mime_type().to_string());
        }
    }

    // Fall back to extension-based guess.
    mime_guess::from_path(path).first().map(|m| m.to_string())
}
