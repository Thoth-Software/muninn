//! Email parser — .eml (RFC 822) via `mailparse`.

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;
use std::path::Path;

pub struct EmlParser;

impl FormatParser for EmlParser {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        _text_extraction_depth: usize,
    ) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: Parse with mailparse::parse_mail
        // TODO: Extract From, To, Date, Subject, MIME structure, attachment list
        // TODO: Body text for language detection
        let _ = (path, doc);
        errors
    }
}
