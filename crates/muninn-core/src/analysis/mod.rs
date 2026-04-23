//! Post-extraction analysis passes.
//!
//! These run after format-specific parsing and operate on extracted text
//! or filesystem metadata to produce derived fields.

pub mod department;
pub mod jargon;
pub mod language;
pub mod summary;
pub mod version;
pub mod xref;
