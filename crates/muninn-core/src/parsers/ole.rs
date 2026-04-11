//! OLE compound document parser — deep inspection for legacy `.doc`, `.xls`, `.ppt`, `.msg`.
//!
//! Uses the `cfb` crate to read OLE streams and extract summary properties.

use std::path::Path;
use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;

pub struct OleParser;

impl FormatParser for OleParser {
    fn parse(&self, path: &Path, doc: &mut DocumentMetadata, text_extraction_depth: usize) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: Open with cfb::CompoundFile::open(path)
        // TODO: Read \x05SummaryInformation stream → author, title, created, modified
        // TODO: For .msg: extract sender, recipients, subject, date, attachment catalog
        let _ = (path, doc, text_extraction_depth);
        errors
    }
}
