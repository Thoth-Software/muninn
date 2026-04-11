//! HTML parser — extract <meta> tags, <title>, structural counts.

use std::path::Path;
use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;

pub struct HtmlParser;

impl FormatParser for HtmlParser {
    fn parse(&self, path: &Path, doc: &mut DocumentMetadata, _text_extraction_depth: usize) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: Extract <meta> author, description, keywords, generator
        // TODO: Extract <title>
        // TODO: Count headings, tables, images, links
        // TODO: Body text for language detection
        let _ = (path, doc);
        errors
    }
}
