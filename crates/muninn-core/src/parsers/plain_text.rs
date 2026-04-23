//! Plain text parser — encoding detection, line/char counts, schema detection
//! for structured formats (CSV, JSON, XML, YAML).

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;
use std::path::Path;

pub struct PlainTextParser;

impl FormatParser for PlainTextParser {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        _text_extraction_depth: usize,
    ) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: Read first N bytes, detect encoding with chardetng
        // TODO: Line count, character count
        // TODO: For csv/tsv: column count, row count
        // TODO: For xml/json/yaml: root element / top-level keys
        // TODO: Language detection from content
        let _ = (path, doc);
        errors
    }
}
