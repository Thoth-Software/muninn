//! # Muninn Core
//!
//! Filesystem metadata scanner for corpus analysis. Walks directory trees,
//! classifies files by inspection depth (deep / medium / shallow / excluded),
//! dispatches to format-specific parsers, and produces a structured JSON report.
//!
//! ## Architecture
//!
//! ```text
//! ScanConfig ──► Scanner ──► FileClassifier ──► ParserDispatch ──► DocumentMetadata
//!                  │                                                      │
//!                  │              (parallel via rayon)                     │
//!                  └──────────────────────────────────────────────────────►│
//!                                                                         ▼
//!                                                              CorpusSummary + Report
//! ```
//!
//! The [`Scanner`] is the top-level entry point. Give it a [`ScanConfig`] and
//! call [`Scanner::run`] to get a [`ScanReport`].

pub mod config;
pub mod error;
pub mod scanner;
pub mod classify;
pub mod parsers;
pub mod analysis;
pub mod output;

// ── Public API re-exports ──────────────────────────────────────────────────

pub use config::ScanConfig;
pub use error::{MuninnError, Result};
pub use output::ScanReport;
pub use scanner::Scanner;
