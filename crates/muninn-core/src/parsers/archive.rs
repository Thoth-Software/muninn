//! Archive / container parser — list contents without full extraction.

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;
use std::path::Path;

pub struct ArchiveParser;

impl FormatParser for ArchiveParser {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        _text_extraction_depth: usize,
    ) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: For zip: list entries (filenames, sizes, count)
        // TODO: Detect "actually a document" cases (OOXML = zip)
        // TODO: Report contained file format distribution
        // TODO: For mbox: count messages, extract date range
        let _ = (path, doc);
        errors
    }
}
