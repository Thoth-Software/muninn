//! Shared helpers for integration tests.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use muninn_core::output::{CorpusSummary, DateInfo, DocumentMetadata, ScanMetadata, ScanReport};

/// Construct a minimal, fully deterministic `ScanReport` for snapshot and
/// schema-validation tests. All timestamps and scalars are hardcoded.
pub fn minimal_scan_report() -> ScanReport {
    let ts: DateTime<Utc> = "2026-01-15T12:00:00Z".parse().unwrap();

    let mut format_distribution = HashMap::new();
    format_distribution.insert("txt".to_string(), 1);

    ScanReport {
        scan_metadata: ScanMetadata {
            scanner_version: "0.1.0".to_string(),
            scan_timestamp: ts,
            scan_roots: vec!["test/root".to_string()],
            scan_duration_seconds: 1.5,
            hostname: Some("test-host".to_string()),
            os: Some("test-os".to_string()),
        },
        corpus_summary: CorpusSummary {
            total_documents: 1,
            total_size_bytes: 1024,
            format_distribution,
            language_distribution: None,
            encoding_distribution: None,
            scanned_vs_digital: None,
            date_modified_distribution: None,
        },
        documents: vec![DocumentMetadata {
            relative_path: "notes.txt".to_string(),
            scan_root: "test/root".to_string(),
            file_size_bytes: 1024,
            extension: "txt".to_string(),
            mime_type: None,
            dates: DateInfo {
                filesystem_created: Some(ts),
                filesystem_modified: Some(ts),
                document_created: None,
                document_modified: None,
            },
            authorship: None,
            version: None,
            encoding: None,
            encoding_confidence: None,
            language: None,
            language_confidence: None,
            structure: None,
            pdf_specific: None,
            inferred_department: None,
            cross_references: None,
            domain_terms: None,
            errors: None,
        }],
    }
}
