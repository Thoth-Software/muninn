//! Post-extraction analysis passes.
//!
//! These run after format-specific parsing and operate on extracted text
//! or filesystem metadata to produce derived fields.

pub mod xref;
pub mod version;
pub mod language;
pub mod jargon;
pub mod department;
pub mod summary;
