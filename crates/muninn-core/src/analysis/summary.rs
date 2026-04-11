//! Corpus summary aggregation.
//!
//! After all documents are scanned, this module walks the document list and
//! computes the aggregate statistics for [`crate::output::CorpusSummary`].

use std::collections::HashMap;

use chrono::Utc;

use crate::output::{
    CorpusSummary, DateModifiedDistribution, DocumentMetadata, ScannedVsDigital,
    TextExtractability,
};

/// Aggregate per-document metadata into a corpus summary.
pub fn compute_summary(documents: &[DocumentMetadata]) -> CorpusSummary {
    let total_documents = documents.len() as u64;
    let total_size_bytes: u64 = documents.iter().map(|d| d.file_size_bytes).sum();

    // Format distribution
    let mut format_distribution: HashMap<String, u64> = HashMap::new();
    for doc in documents {
        *format_distribution.entry(doc.extension.clone()).or_default() += 1;
    }

    // Language distribution
    let mut lang_dist: HashMap<String, u64> = HashMap::new();
    for doc in documents {
        if let Some(ref lang) = doc.language {
            *lang_dist.entry(lang.clone()).or_default() += 1;
        }
    }
    let language_distribution = if lang_dist.is_empty() { None } else { Some(lang_dist) };

    // Encoding distribution
    let mut enc_dist: HashMap<String, u64> = HashMap::new();
    for doc in documents {
        if let Some(ref enc) = doc.encoding {
            *enc_dist.entry(enc.clone()).or_default() += 1;
        }
    }
    let encoding_distribution = if enc_dist.is_empty() { None } else { Some(enc_dist) };

    // Scanned vs digital (PDFs only)
    let scanned_vs_digital = compute_scanned_vs_digital(documents);

    // Date modified distribution
    let date_modified_distribution = compute_date_distribution(documents);

    CorpusSummary {
        total_documents,
        total_size_bytes,
        format_distribution,
        language_distribution,
        encoding_distribution,
        scanned_vs_digital,
        date_modified_distribution,
    }
}

fn compute_scanned_vs_digital(documents: &[DocumentMetadata]) -> Option<ScannedVsDigital> {
    let mut born_digital = 0u64;
    let mut scanned = 0u64;
    let mut mixed = 0u64;
    let mut any = false;

    for doc in documents {
        if let Some(ref pdf) = doc.pdf_specific {
            if let Some(ref ext) = pdf.text_extractability {
                any = true;
                match ext {
                    TextExtractability::BornDigital => born_digital += 1,
                    TextExtractability::Scanned => scanned += 1,
                    TextExtractability::Mixed => mixed += 1,
                }
            }
        }
    }

    if any {
        Some(ScannedVsDigital { born_digital, scanned, mixed })
    } else {
        None
    }
}

fn compute_date_distribution(documents: &[DocumentMetadata]) -> Option<DateModifiedDistribution> {
    let now = Utc::now();
    let mut last_30 = 0u64;
    let mut last_90 = 0u64;
    let mut last_365 = 0u64;
    let mut older = 0u64;

    for doc in documents {
        if let Some(modified) = doc.dates.filesystem_modified {
            let age = now.signed_duration_since(modified);
            if age.num_days() <= 30 {
                last_30 += 1;
            } else if age.num_days() <= 90 {
                last_90 += 1;
            } else if age.num_days() <= 365 {
                last_365 += 1;
            } else {
                older += 1;
            }
        }
    }

    Some(DateModifiedDistribution {
        last_30_days: last_30,
        last_90_days: last_90,
        last_365_days: last_365,
        older,
    })
}
