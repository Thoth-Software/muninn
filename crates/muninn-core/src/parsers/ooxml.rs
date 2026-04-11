//! OOXML parser — deep inspection for `.docx`, `.xlsx`, `.pptx` (and macro variants).
//!
//! OOXML files are zip archives. Unzip, parse `docProps/core.xml` for authorship
//! and dates, then format-specific structural extraction from the document XML.

use std::path::Path;

use crate::error::ExtractionError;
use crate::output::DocumentMetadata;
use crate::parsers::FormatParser;

pub struct OoxmlParser;

impl FormatParser for OoxmlParser {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        text_extraction_depth: usize,
    ) -> Vec<ExtractionError> {
        let mut errors = Vec::new();

        // TODO: Open with zip::ZipArchive
        // TODO: Parse docProps/core.xml with quick-xml:
        //       - dc:creator → authorship.author
        //       - cp:lastModifiedBy → authorship.last_modified_by
        //       - dcterms:created → dates.document_created
        //       - dcterms:modified → dates.document_modified
        //       - dc:title, dc:subject, cp:keywords, cp:revision
        //
        // TODO: Format-specific structural extraction:
        //   DOCX: parse document.xml → heading count/depth, table count, image count,
        //         footnote/endnote count, list count/nesting, comment count
        //   XLSX: sheet count, named ranges, row/column dimensions per sheet
        //   PPTX: slide count, notes slide count, embedded image count
        //
        // TODO: Text extraction from content XML for language detection

        let _ = (path, doc, text_extraction_depth);
        errors
    }
}
