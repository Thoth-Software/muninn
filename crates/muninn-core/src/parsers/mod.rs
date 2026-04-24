//! Format-specific parsers.
//!
//! Each parser implements the [`FormatParser`] trait, which takes a file path
//! and returns extraction errors along with whatever metadata that format can
//! yield. The scanner dispatches to the correct parser based on the
//! [`crate::classify::FileClassification`].
//!
//! Parsers are feature-gated: disabling the `pdf` feature removes the PDF
//! parser and its `lopdf` / `pdf-extract` dependencies entirely.

use std::path::Path;

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;

pub mod common;

#[cfg(feature = "pdf")]
pub mod pdf;

#[cfg(feature = "ooxml")]
pub mod ooxml;

#[cfg(feature = "ole")]
pub mod ole;

pub mod html;
pub mod plain_text;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "image-meta")]
pub mod image;

pub mod archive;
pub mod opendocument;
pub mod rtf;

/// Trait implemented by every format-specific parser.
///
/// Parsers receive a partially-populated [`DocumentMetadata`] (filesystem
/// fields already filled in by the scanner) and enrich it with format-
/// specific data. They return a list of non-fatal errors encountered
/// during extraction.
pub trait FormatParser: Send + Sync {
    /// Enrich `doc` with format-specific metadata extracted from `path`.
    ///
    /// Returns any non-fatal errors encountered. A parser should *not*
    /// return `Err` for recoverable issues — instead, push an
    /// [`ExtractionError`] and continue extracting what it can.
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        text_extraction_depth: usize,
    ) -> Vec<ExtractionError>;
}
