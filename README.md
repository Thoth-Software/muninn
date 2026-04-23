# Muninn

Filesystem metadata scanner for corpus analysis. Part of the Cake Intelligence platform.

Muninn walks directory trees on a customer's machine, classifies every file by how deeply it can be inspected, dispatches to format-specific parsers, runs analysis passes over extracted text, and produces a single structured JSON report. That report is what Cake ingests to configure its RAG pipeline for the customer's corpus. Muninn does not perform that analysis itself — it extracts and exports. It does not connect to APIs, require authentication, or phone home. The signed `.exe` and its JSON output are the entire surface area.

The near-term target customer is AmeriWater.

---

## Architecture

### Pipeline

Every scan follows the same linear pipeline:

```
ScanConfig
    │
    ▼
Scanner::run()
    │
    ├─ Validate scan roots exist and are accessible
    ├─ Compile exclusion GlobSet from config
    ├─ Compile cross-reference RegexSet from defaults + custom patterns
    │
    ├─ walkdir each scan root
    │   ├─ skip excluded paths (glob match)
    │   ├─ skip files exceeding max_file_size_bytes
    │   └─ collect (absolute_path, scan_root) tuples
    │
    ├─ rayon::par_iter over collected file paths
    │   │
    │   └─ process_file():
    │       ├─ common::extract_filesystem_metadata()       ← ALL files get this
    │       ├─ classify::classify_by_extension()            ← dispatch table lookup
    │       ├─ get_parser(kind) → FormatParser::parse()     ← deep + medium only
    │       ├─ analysis::version::detect_version()          ← filename pattern matching
    │       ├─ analysis::department::infer_department()     ← directory path heuristics
    │       └─ if extracted text available:
    │           ├─ analysis::xref::extract()                ← regex cross-references
    │           ├─ analysis::language::detect_language()     ← ISO 639-1 classification
    │           └─ analysis::jargon::extract_jargon_terms() ← TF-IDF vs background corpus
    │
    ├─ analysis::summary::compute_summary()                 ← aggregate all documents
    │
    └─ Assemble ScanReport { scan_metadata, corpus_summary, documents }
        └─ Serialize to JSON, write to output_dir
```

There is no async. This is CPU+I/O bound batch processing over a local filesystem, parallelized with rayon. The pipeline is not a server, not a daemon, not long-running — it starts, scans, writes a file, and exits.

### Inspection Depth

Every file gets classified into one of four tiers. The tier determines how much work Muninn does on it.

| Tier | What happens | Examples |
|---|---|---|
| **Deep** | Filesystem metadata + internal document metadata + structural features + text extraction | `.pdf`, `.docx`, `.xlsx`, `.pptx`, `.doc`, `.xls`, `.ppt`, `.odt`, `.html`, `.eml`, `.csv`, `.json`, `.xml`, `.md`, `.txt` |
| **Medium** | Filesystem metadata + container/header metadata, no full text extraction | `.zip`, `.7z`, `.dwg`, `.step`, `.ifc`, `.png`, `.jpg`, `.tiff`, `.svg`, `.epub`, `.sqlite` |
| **Shallow** | Filesystem metadata only (size, dates, MIME detection) | `.mp3`, `.mp4`, `.exe`, `.dll`, `.ttf`, `.woff` |
| **Excluded** | Skipped entirely, not even counted | `Thumbs.db`, `.DS_Store`, `.git/`, `node_modules/`, `*.tmp`, `*.pyc` |

The dispatch table lives in `classify.rs`. It maps file extensions to `(InspectionDepth, ParserKind)` pairs. Magic-byte MIME detection via the `infer` crate catches mismatched extensions.

### Parser Architecture

Every format-specific parser implements the `FormatParser` trait:

```rust
pub trait FormatParser: Send + Sync {
    fn parse(
        &self,
        path: &Path,
        doc: &mut DocumentMetadata,
        text_extraction_depth: usize,
    ) -> Vec<ExtractionError>;
}
```

The contract:

- Parsers receive a `DocumentMetadata` with filesystem fields already populated (by `common.rs`). They enrich it with format-specific data.
- Parsers are **non-fatal**. A corrupt PDF must not abort a 50,000-file scan. If something goes wrong, push an `ExtractionError` with a `stage` string and `message` onto the return vec. The scanner stores these in the document's `errors` array. Never `panic!`, never return `Err` for a single-file problem.
- `text_extraction_depth` controls how many pages/slides/sheets to sample for language detection, cross-reference extraction, and jargon analysis. Default: 5.
- Parsers are `Send + Sync` because they run in rayon's thread pool.

Parser modules are feature-gated. Disabling a feature removes the parser and its dependencies from the compile entirely:

| Feature | Parser module | Crates gated |
|---|---|---|
| `pdf` | `parsers::pdf` | `lopdf`, `pdf-extract` |
| `ooxml` | `parsers::ooxml` | `zip`, `quick-xml` |
| `ole` | `parsers::ole` | `cfb` |
| `email` | `parsers::email` | `mailparse` |
| `image-meta` | `parsers::image` | `kamadak-exif`, `imagesize` |
| `text-analysis` | `analysis::xref` | `aho-corasick` |
| `lang-detect` | `analysis::language` | `whichlang` |

Parsers that don't pull heavy dependencies (`plain_text`, `html`, `rtf`, `opendocument`, `archive`) are always compiled.

### Error Model

Two distinct error types, used for different purposes:

**`MuninnError`** (thiserror enum) — scan-level failures that affect control flow. Variants: `ScanRootNotFound`, `PermissionDenied`, `FileTooLarge`, `ParseError`, `Io`, `Json`, `Regex`, `Glob`, `WalkDir`. The scanner's `run()` method returns `Result<ScanReport, MuninnError>`. The CLI catches this and exits non-zero.

**`ExtractionError`** (serde struct) — per-document extraction failures that are data, not control flow. Fields: `stage` (which extraction step failed) and `message`. These are recorded in the document's `errors` array in the output JSON. The scan continues.

The boundary is: if one file is broken, that's an `ExtractionError`. If the scan root doesn't exist, that's a `MuninnError`.

### Output Schema

The output is a single JSON file conforming to `muninn-output-schema.json` (JSON Schema draft-07). The Rust types in `output.rs` map 1:1 to this schema. Three top-level objects:

**`scan_metadata`** — information about the scan itself: scanner version, timestamp, scan roots, duration, hostname, OS.

**`corpus_summary`** — aggregate statistics computed from all documents: total count, total size, format distribution, language distribution, encoding distribution, scanned-vs-digital breakdown (PDFs only), date-modified distribution by recency bucket.

**`documents`** — array of per-document metadata. One entry per file. Contains: relative path, file size, extension, MIME type, dates (filesystem + document-internal), authorship, version info (filename pattern), encoding, language, structural features (headings, tables, images, lists, page count, complexity tier), PDF-specific fields (text extractability, form detection), inferred department, cross-references, domain-specific jargon terms, and any extraction errors.

### Input Model

Mapped in `config.rs`. Three tiers of input, matching the GUI design:

**Main GUI** (visible to everyone): scan roots (mandatory), output location (defaults to Desktop), consent toggles (include hostname, include full paths).

**Advanced panel** (collapsed by default, IT-facing): exclusion patterns (glob), max file size threshold, text extraction depth, custom cross-reference regex patterns, department-to-directory mapping overrides, scan concurrency.

**Baked into the binary** (developer config): default exclusion list, supported format registry, language detection model weights, version pattern regexes, default cross-reference patterns, jargon background corpus, complexity decision tree model (placeholder — not trained yet), scanner version string.

### Distribution Model

Muninn is distributed as a signed Windows `.exe`. A dual-audience trust model shapes the design:

- **Executive-level users** (the person who runs it): code signing addresses trust. Minimal GUI — folder picker, output location, consent toggles, scan button.
- **IT/security team** (the people who approve it): public GitHub repo for source auditing. Feature-gated dependencies so they can verify exactly what runs.

---

## Workspace Layout

```
muninn/
├── Cargo.toml                          # Virtual manifest — no [package], just [workspace]
├── CLAUDE.md                           # Operational contract for AI sessions
├── README.md                           # This file — architecture reference
├── docs/
│   └── reference/
│       └── rust-best-practices.md      # Rust idioms and patterns reference
├── crates/
│   ├── muninn-core/                    # Library crate: all scanner logic
│   │   ├── Cargo.toml                  # Feature flags gate format parsers
│   │   └── src/
│   │       ├── lib.rs                  # Public API re-exports: ScanConfig, Scanner, ScanReport, MuninnError, Result
│   │       ├── error.rs                # MuninnError (thiserror) + ExtractionError (serde data type)
│   │       ├── config.rs              # ScanConfig — maps to Input Specification
│   │       ├── scanner.rs              # Pipeline orchestrator
│   │       ├── classify.rs             # Extension → (InspectionDepth, ParserKind) dispatch table
│   │       ├── output.rs              # Report data types — maps to muninn-output-schema.json
│   │       ├── parsers/
│   │       │   ├── mod.rs              # FormatParser trait
│   │       │   ├── common.rs           # Filesystem metadata (runs for every file)
│   │       │   ├── pdf.rs              # [feature: pdf] lopdf + pdf-extract
│   │       │   ├── ooxml.rs            # [feature: ooxml] zip + quick-xml (docx, xlsx, pptx)
│   │       │   ├── ole.rs              # [feature: ole] cfb (doc, xls, ppt, msg)
│   │       │   ├── plain_text.rs       # Always compiled (txt, csv, json, xml, yaml, md, etc.)
│   │       │   ├── html.rs             # Always compiled
│   │       │   ├── rtf.rs              # Always compiled
│   │       │   ├── opendocument.rs     # Always compiled (odt, ods, odp)
│   │       │   ├── email.rs            # [feature: email] mailparse (eml)
│   │       │   ├── image.rs            # [feature: image-meta] kamadak-exif + imagesize
│   │       │   └── archive.rs          # Always compiled (zip listing, mbox counting)
│   │       └── analysis/
│   │           ├── mod.rs
│   │           ├── xref.rs             # Cross-reference extraction (RegexSet)
│   │           ├── version.rs          # Filename version pattern detection
│   │           ├── language.rs         # [feature: lang-detect] whichlang
│   │           ├── jargon.rs           # TF-IDF against baked-in background corpus
│   │           ├── department.rs       # Directory path → department inference
│   │           └── summary.rs          # Corpus-level aggregation
│   ├── muninn-cli/                     # Binary crate: thin CLI shell
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                 # clap derive → ScanConfig → Scanner::run → write JSON
│   └── muninn-gui/                     # Binary crate: eframe/egui GUI (not yet active)
├── resources/                          # Baked-in data (language model, jargon background corpus)
└── tests/
    └── fixtures/                       # Sample files for integration tests
```

### Module Responsibilities

| Module | Responsibility | Does NOT do |
|---|---|---|
| `config.rs` | Hold scan configuration, provide defaults | Validate paths, compile patterns |
| `scanner.rs` | Orchestrate the pipeline, manage parallelism | Parse any file format directly |
| `classify.rs` | Map extensions to inspection depth + parser kind | MIME detection (that's in `common.rs`) |
| `parsers::common` | Extract filesystem metadata, detect MIME type | Format-specific parsing |
| `parsers::pdf` | Extract PDF internals via lopdf/pdf-extract | Decide what to do with the metadata |
| `parsers::ooxml` | Extract OOXML internals via zip/quick-xml | Handle legacy OLE formats |
| `output.rs` | Define the report data model, serialize to JSON | Compute summary statistics |
| `analysis::summary` | Aggregate per-document data into corpus summary | Per-document extraction |
| `analysis::xref` | Match regex patterns against extracted text | Decide which patterns to use (that's config) |
| `analysis::version` | Match filename patterns, compute version families | Anything involving file contents |
| `analysis::department` | Infer department from directory path | Anything involving file contents |

---

## Spec Documents

These are the source-of-truth design documents. When code and spec disagree, the spec wins — flag the discrepancy rather than silently following the code.

| Document | What it governs |
|---|---|
| **Muninn: Input Specification** | Everything the scanner takes as input, organized by who provides it |
| **Muninn: Supported File Formats** | The full format registry (~130 extensions) with inspection depths and extraction details |
| **muninn-output-schema.json** | JSON Schema for the report; `output.rs` types must match exactly |
| **muninn-crates.md** | Dependency rationale for each crate |

---

## Tech Stack

| Concern | Choice | Rationale |
|---|---|---|
| Language | Rust 2021, MSRV 1.75 | Single-binary distribution, no runtime dependencies on customer machines |
| Concurrency | rayon | Data-parallel batch processing, not async I/O |
| Error handling | `thiserror` (library), `anyhow` (binary) | Structured matching in core, ergonomic propagation in CLI |
| Serialization | serde + serde_json, chrono | JSON report output, ISO 8601 timestamps |
| CLI | clap v4 (derive) | Ergonomic arg parsing with help generation |
| GUI (future) | eframe / egui | Single-binary, no web runtime, native on Windows |
| Logging | tracing + tracing-subscriber | Structured, filterable |
| Filesystem | walkdir, filetime, mime_guess, infer, globset | Each does one thing well |
| PDF | lopdf, pdf-extract | Metadata + text extraction |
| OOXML | zip, quick-xml | docx/xlsx/pptx are zip archives with XML |
| OLE | cfb | Legacy doc/xls/ppt/msg |
| Text/encoding | chardetng, encoding_rs | Encoding detection + conversion |
| Language | whichlang | Pure Rust, lightweight, no model file |
| Regex | regex, aho-corasick | Cross-references, version patterns, jargon |
| Email | mailparse | RFC 822 .eml parsing |
| Image | kamadak-exif, imagesize | EXIF + dimensions |

---

## Static Analysis

| Tool | Purpose | Install |
|---|---|---|
| `cargo clippy` | Lint — correctness, style, performance | Ships with rustup |
| `cargo fmt` | Format — consistent style via rustfmt | Ships with rustup |
| `cargo deny` | Audit — license compliance, advisory database, duplicate deps | `cargo install cargo-deny` |
| `cargo audit` | Security — RustSec advisory database check | `cargo install cargo-audit` |
| `cargo machete` | Unused deps — declared but not imported | `cargo install cargo-machete` |
| `cargo outdated` | Staleness — shows deps with newer versions | `cargo install cargo-outdated` |

---

## Testing Libraries

| Library | Purpose | Scope |
|---|---|---|
| Built-in `#[test]` | Unit tests, doc tests | All crates |
| `tempfile` | Temporary directories/files for filesystem tests | `muninn-core` dev-deps |
| `assert_cmd` | CLI integration tests — run binary, assert exit/stdout/stderr | `muninn-cli` dev-deps |
| `predicates` | Assertion helpers for assert_cmd | `muninn-cli` dev-deps |
| `proptest` | Property-based / fuzz testing for parsers | `muninn-core` dev-deps (to add) |
| `insta` | Snapshot testing — golden-file comparison for JSON output | `muninn-core` dev-deps (to add) |
| `jsonschema` | Schema validation — verify output conforms to schema | `muninn-core` dev-deps (to add) |
| `criterion` | Benchmarking — parser throughput on large corpora | Workspace dev-deps (to add) |
