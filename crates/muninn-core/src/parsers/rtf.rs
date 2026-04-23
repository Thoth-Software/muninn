//! RTF parser — metadata from header fields, text extraction by stripping control words.

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;
use std::path::Path;

pub struct RtfParser;

impl FormatParser for RtfParser {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        _text_extraction_depth: usize,
    ) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: Parse \info group for \author, \title, \creatim
        // TODO: Strip RTF control words for language detection
        let _ = (path, doc);
        errors
    }
}
