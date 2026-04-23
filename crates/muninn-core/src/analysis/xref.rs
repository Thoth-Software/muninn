//! Cross-reference extraction from document text.
//!
//! Matches baked-in patterns (ISO, ANSI, ASTM, CFR, generic "See X") plus
//! user-provided custom patterns against extracted text content.

use regex::RegexSet;

use crate::output::{CrossReference, ReferenceType};

/// Compiled pattern set, built once from defaults + user custom patterns.
pub struct XrefExtractor {
    // TODO: RegexSet for fast multi-pattern matching
    // TODO: Mapping from pattern index → ReferenceType classification
    _patterns: RegexSet,
}

impl XrefExtractor {
    /// Build from default + custom patterns.
    pub fn new(custom_patterns: &[String]) -> crate::Result<Self> {
        let mut all_patterns: Vec<&str> = vec![
            // ISO standards
            r"ISO\s*\d{3,5}(-\d+)?",
            // ANSI standards
            r"ANSI[/ ][\w.\-]+",
            // ASTM standards
            r"ASTM\s*[A-Z]\d{1,4}",
            // CFR references
            r"\d+\s*CFR\s*§?\s*\d+(\.\d+)?",
            // Generic "See Section/Appendix/Exhibit"
            r"[Ss]ee\s+(Section|Appendix|Exhibit|Table|Figure)\s+[\w.\-]+",
            // Generic "per Document N"
            r"[Pp]er\s+[A-Z][\w.\-]+\s+\d+",
        ];

        let custom_strs: Vec<&str> = custom_patterns.iter().map(|s| s.as_str()).collect();
        all_patterns.extend(custom_strs);

        let patterns = RegexSet::new(&all_patterns)?;
        Ok(Self {
            _patterns: patterns,
        })
    }

    /// Extract cross-references from a block of text.
    pub fn extract(&self, _text: &str) -> Vec<CrossReference> {
        // TODO: Match _patterns against text
        // TODO: For each match, classify as Standard / InternalId / External / Unknown
        // TODO: Deduplicate
        Vec::new()
    }
}

/// Classify a matched reference string into a [`ReferenceType`].
fn _classify_reference(_matched: &str) -> ReferenceType {
    // TODO: ISO/ANSI/ASTM/CFR → Standard
    // TODO: User-provided patterns → InternalId
    // TODO: URLs → External
    // TODO: Fallback → Unknown
    ReferenceType::Unknown
}
