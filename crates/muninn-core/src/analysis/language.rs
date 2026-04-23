//! Language detection from extracted text.
//!
//! Uses `whichlang` (pure Rust, lightweight) when the `lang-detect` feature
//! is enabled. Falls back to `None` otherwise.

/// Detect the primary language of a text sample.
/// Returns (ISO 639-1 code, confidence 0.0–1.0).
#[must_use]
pub fn detect_language(_text: &str) -> Option<(String, f64)> {
    #[cfg(feature = "lang-detect")]
    {
        // TODO: Use whichlang to classify
        // TODO: Map whichlang's Lang enum to ISO 639-1 codes
        // TODO: whichlang doesn't provide confidence scores natively —
        //       may need to use text length as a rough proxy, or switch
        //       to fasttext if accuracy matters more than binary size.
        None
    }
    #[cfg(not(feature = "lang-detect"))]
    {
        None
    }
}
