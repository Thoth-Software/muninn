//! `OpenDocument` parser — .odt, .ods, .odp. Unzip + parse meta.xml.

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;
use std::path::Path;

pub struct OpenDocumentParser;

impl FormatParser for OpenDocumentParser {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        _text_extraction_depth: usize,
    ) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: Unzip, parse meta.xml for author, dates, editing cycles, generator
        // TODO: odt: heading count, table count, page count from document statistics
        // TODO: ods: sheet count
        // TODO: odp: slide count
        // TODO: Text from content.xml for language detection
        let _ = (path, doc);
        errors
    }
}
