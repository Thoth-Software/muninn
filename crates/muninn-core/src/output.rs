//! Output types — the data model for the Muninn scan report.
//!
//! These structs serialize directly to the JSON schema defined in
//! `muninn-output-schema.json`. Field names use `#[serde(rename)]` where
//! the JSON uses snake_case that differs from Rust conventions.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// Top-level report
// ═══════════════════════════════════════════════════════════════════════════

/// One report per scan run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub scan_metadata: ScanMetadata,
    pub corpus_summary: CorpusSummary,
    pub documents: Vec<DocumentMetadata>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Scan metadata
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub scanner_version: String,
    pub scan_timestamp: DateTime<Utc>,
    pub scan_roots: Vec<String>,
    pub scan_duration_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Corpus summary
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusSummary {
    pub total_documents: u64,
    pub total_size_bytes: u64,
    pub format_distribution: HashMap<String, u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_distribution: Option<HashMap<String, u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_distribution: Option<HashMap<String, u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scanned_vs_digital: Option<ScannedVsDigital>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_modified_distribution: Option<DateModifiedDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedVsDigital {
    pub born_digital: u64,
    pub scanned: u64,
    pub mixed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateModifiedDistribution {
    pub last_30_days: u64,
    pub last_90_days: u64,
    pub last_365_days: u64,
    pub older: u64,
}

// ═══════════════════════════════════════════════════════════════════════════
// Per-document metadata
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub relative_path: String,
    pub scan_root: String,
    pub file_size_bytes: u64,
    pub extension: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    pub dates: DateInfo,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorship: Option<Authorship>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<VersionInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_confidence: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_confidence: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure: Option<StructureInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_specific: Option<PdfSpecific>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inferred_department: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_references: Option<Vec<CrossReference>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_terms: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<crate::error::ExtractionError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem_created: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem_modified: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_created: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorship {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string extracted from filename pattern (e.g. "v2", "rev3", "FINAL").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename_version: Option<String>,
    /// Base filename with version stripped, for clustering version families.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_max_depth: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnote_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_max_nesting_depth: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_reference_count: Option<u64>,
    /// Classified by decision tree if model is available, otherwise null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity_tier: Option<ComplexityTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityTier {
    Flat,
    Moderate,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfSpecific {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_extractability: Option<TextExtractability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages_with_text: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages_without_text: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_image_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_forms: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextExtractability {
    BornDigital,
    Scanned,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub reference_text: String,
    pub reference_type: ReferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    InternalId,
    Standard,
    External,
    Unknown,
}
