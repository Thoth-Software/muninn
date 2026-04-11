//! Domain-specific jargon detection via TF-IDF comparison.
//!
//! Compares term frequencies in a document against a background corpus
//! (general English word frequencies, baked into the binary) to surface
//! terms that are unusually frequent in the document — likely domain jargon.

/// Extract candidate jargon terms from document text.
///
/// Returns terms sorted by TF-IDF score descending.
pub fn extract_jargon_terms(_text: &str) -> Vec<String> {
    // TODO: Tokenize text, compute term frequencies
    // TODO: Load baked-in background corpus (top 50k English words)
    // TODO: Compute TF-IDF: tf(term, doc) * log(N / df(term, background))
    // TODO: Return top N terms above threshold
    Vec::new()
}
