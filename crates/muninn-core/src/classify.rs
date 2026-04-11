//! File classification.
//!
//! The classifier is the internal dispatch table that determines what Muninn
//! does when it encounters a given file. Every file gets at least shallow
//! inspection (filesystem metadata + MIME detection). The inspection depth
//! determines how much further Muninn goes.

use std::path::Path;

/// How deeply Muninn inspects a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InspectionDepth {
    /// Full metadata + structural features + text extraction.
    Deep,
    /// Container metadata + content catalog, no full text analysis.
    Medium,
    /// Filesystem metadata only (size, dates, MIME).
    Shallow,
    /// Skip entirely — not even counted.
    Excluded,
}

/// Which parser module handles a given format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserKind {
    Pdf,
    Ooxml,       // docx, xlsx, pptx (and macro variants)
    Ole,         // doc, xls, ppt, msg
    Rtf,
    OpenDocument, // odt, ods, odp
    PlainText,   // txt, md, csv, json, xml, yaml, etc.
    Html,
    Eml,
    ArchiveContainer, // zip, 7z, tar, etc.
    CadDxf,
    CadStep,
    CadIfc,
    Image,
    Ebook,
    Database,
    /// No parser — filesystem metadata only.
    None,
}

/// Classification result for a single file.
#[derive(Debug, Clone)]
pub struct FileClassification {
    pub depth: InspectionDepth,
    pub parser: ParserKind,
}

/// Classify a file by its extension (lowercase, no dot).
///
/// Falls back to [`InspectionDepth::Shallow`] with [`ParserKind::None`] for
/// unrecognized extensions. Magic-byte MIME detection happens later in the
/// pipeline and can upgrade a shallow classification.
pub fn classify_by_extension(path: &Path) -> FileClassification {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        // ── Deep: PDF ──────────────────────────────────────────────
        "pdf" => deep(ParserKind::Pdf),

        // ── Deep: OOXML ────────────────────────────────────────────
        "docx" | "docm" => deep(ParserKind::Ooxml),
        "xlsx" | "xlsm" => deep(ParserKind::Ooxml),
        "pptx" | "pptm" => deep(ParserKind::Ooxml),

        // ── Deep: Legacy Office (OLE) ──────────────────────────────
        "doc" => deep(ParserKind::Ole),
        "xls" => deep(ParserKind::Ole),
        "ppt" => deep(ParserKind::Ole),

        // ── Deep: RTF ──────────────────────────────────────────────
        "rtf" => deep(ParserKind::Rtf),

        // ── Deep: OpenDocument ─────────────────────────────────────
        "odt" | "ods" | "odp" => deep(ParserKind::OpenDocument),

        // ── Deep: Plain text family ────────────────────────────────
        "txt" | "text" | "log" | "md" | "markdown" | "rst" | "csv"
        | "tsv" | "json" | "jsonl" | "xml" | "yaml" | "yml"
        | "toml" | "ini" | "cfg" | "conf" => deep(ParserKind::PlainText),

        // ── Deep: HTML ─────────────────────────────────────────────
        "html" | "htm" | "xhtml" | "mhtml" | "mht" => deep(ParserKind::Html),

        // ── Deep: Email ────────────────────────────────────────────
        "eml" => deep(ParserKind::Eml),
        "msg" => deep(ParserKind::Ole), // OLE container with email semantics

        // ── Deep: Email archives (treated as containers) ───────────
        "mbox" | "pst" => medium(ParserKind::ArchiveContainer),

        // ── Medium: Archives ───────────────────────────────────────
        "zip" | "7z" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "rar" => {
            medium(ParserKind::ArchiveContainer)
        }

        // ── Medium: CAD ────────────────────────────────────────────
        "dxf" => medium(ParserKind::CadDxf),
        "dwg" => medium(ParserKind::None), // binary, limited without Autodesk libs
        "step" | "stp" => medium(ParserKind::CadStep),
        "iges" | "igs" => medium(ParserKind::CadStep), // similar text-based header
        "stl" | "3mf" | "obj" => medium(ParserKind::None),
        "ifc" => medium(ParserKind::CadIfc),
        "rvt" | "rfa" => medium(ParserKind::Ole), // Revit = OLE compound

        // ── Medium: Images (EXIF / dimensions) ─────────────────────
        "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp" | "gif"
        | "webp" | "svg" | "ico" => medium(ParserKind::Image),

        // ── Medium: Engineering images ─────────────────────────────
        "vsdx" => medium(ParserKind::Ooxml), // OOXML-like
        "vsd" => medium(ParserKind::Ole),
        "ai" | "eps" | "ps" => medium(ParserKind::None), // DSC comments — future

        // ── Medium: Databases ──────────────────────────────────────
        "sqlite" | "db" | "mdb" | "accdb" => medium(ParserKind::Database),

        // ── Medium: Ebooks ─────────────────────────────────────────
        "epub" | "mobi" => medium(ParserKind::Ebook),

        // ── Shallow: Audio ─────────────────────────────────────────
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => {
            shallow()
        }

        // ── Shallow: Video ─────────────────────────────────────────
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "webm" | "flv" | "m4v" => {
            shallow()
        }

        // ── Shallow: Executables / Binaries ────────────────────────
        "exe" | "dll" | "so" | "dylib" | "bin" | "com" | "msi"
        | "dmg" | "app" => shallow(),

        // ── Shallow: Fonts ─────────────────────────────────────────
        "ttf" | "otf" | "woff" | "woff2" | "eot" => shallow(),

        // ── Unknown extension: default to shallow ──────────────────
        _ => shallow(),
    }
}

fn deep(parser: ParserKind) -> FileClassification {
    FileClassification { depth: InspectionDepth::Deep, parser }
}

fn medium(parser: ParserKind) -> FileClassification {
    FileClassification { depth: InspectionDepth::Medium, parser }
}

fn shallow() -> FileClassification {
    FileClassification { depth: InspectionDepth::Shallow, parser: ParserKind::None }
}
