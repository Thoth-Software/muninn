//! Image metadata parser — EXIF + dimensions.

use std::path::Path;
use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;

pub struct ImageParser;

impl FormatParser for ImageParser {
    fn parse(&self, path: &Path, doc: &mut DocumentMetadata, _text_extraction_depth: usize) -> Vec<ExtractionError> {
        let errors = Vec::new();
        // TODO: kamadak-exif for EXIF (camera, GPS, date taken, software)
        // TODO: imagesize for dimensions
        // TODO: SVG: parse for embedded text, <title>, <metadata>
        let _ = (path, doc);
        errors
    }
}
