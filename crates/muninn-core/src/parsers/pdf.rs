//! PDF parser — deep inspection.
//!
//! Extracts: internal metadata (Author, CreationDate, ModDate, Producer, etc.),
//! page count, embedded image count, form field detection, text extractability
//! classification (born-digital / scanned / mixed), text for language detection
//! and cross-reference extraction.

use std::path::Path;

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;

pub struct PdfParser;

impl FormatParser for PdfParser {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        text_extraction_depth: usize,
    ) -> Vec<ExtractionError> {
        let mut errors = Vec::new();

        // TODO: Open with lopdf::Document::load(path)
        // TODO: Extract /Info dictionary → author, title, subject, keywords, producer, creator
        // TODO: Extract CreationDate, ModDate → doc.dates.document_created / document_modified
        // TODO: Count pages → doc.structure.page_count
        // TODO: Detect form fields (AcroForm) → doc.pdf_specific.has_forms
        // TODO: Per-page text extraction (first `text_extraction_depth` pages) via pdf-extract
        //       - Classify born_digital / scanned / mixed based on text yield per page
        //       - Count pages_with_text / pages_without_text
        // TODO: Count embedded images
        // TODO: Feed extracted text to language detection + cross-reference extraction

        let _ = (path, doc, text_extraction_depth); // suppress unused warnings
        errors
    }
}
