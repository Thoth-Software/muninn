//! Muninn error types.
//!
//! Uses `thiserror` for structured, matchable errors. Callers (CLI, GUI) can
//! inspect variants to decide how to surface failures — e.g. a single file
//! parse error shouldn't abort the whole scan.

use std::path::PathBuf;
use thiserror::Error;

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, MuninnError>;

#[derive(Error, Debug)]
pub enum MuninnError {
    /// A scan root path doesn't exist or isn't accessible.
    #[error("scan root not found: {path}")]
    ScanRootNotFound { path: PathBuf },

    /// Permission denied on a file or directory.
    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    /// A file exceeded the configured max size threshold.
    #[error("file too large ({size_bytes} bytes): {path}")]
    FileTooLarge { path: PathBuf, size_bytes: u64 },

    /// A format-specific parser failed on a single document.
    /// This is non-fatal — the error is recorded in the document's `errors`
    /// array and the scan continues.
    #[error("parse error in {stage} for {path}: {message}")]
    ParseError {
        path: PathBuf,
        stage: String,
        message: String,
    },

    /// Underlying I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error (output writing).
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Regex compilation error (user-provided cross-reference patterns).
    #[error("invalid regex pattern: {0}")]
    Regex(#[from] regex::Error),

    /// Glob pattern error (user-provided exclusion patterns).
    #[error("invalid glob pattern: {0}")]
    Glob(#[from] globset::Error),

    /// walkdir traversal error.
    #[error("directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

/// Per-document extraction error, stored in the output JSON.
/// Distinct from `MuninnError` — this is data, not control flow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionError {
    pub stage: String,
    pub message: String,
}
