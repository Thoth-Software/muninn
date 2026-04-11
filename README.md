# Muninn

Filesystem metadata scanner for corpus analysis. Part of the Cake Intelligence platform.

*Ravens, Hugin and Munin, of Thought and Memory*  
*Wing the wide world each day:*  
*I tremble for Thought, lest he come not again,*  
*Yet for Memory more I fear.*  

## Architecture

Muninn walks directory trees, classifies files by inspection depth (deep / medium / shallow / excluded), dispatches to format-specific parsers, and produces a structured JSON report.

```
muninn/
├── Cargo.toml                    # Workspace root (virtual manifest)
├── crates/
│   ├── muninn-core/              # Library: scanner pipeline + all parsers
│   │   └── src/
│   │       ├── lib.rs            # Public API re-exports
│   │       ├── config.rs         # ScanConfig (maps to Input Specification)
│   │       ├── error.rs          # MuninnError + ExtractionError
│   │       ├── scanner.rs        # Pipeline orchestrator (walk → classify → parse → analyze)
│   │       ├── classify.rs       # Extension → InspectionDepth + ParserKind dispatch table
│   │       ├── output.rs         # Report data types (maps 1:1 to output JSON schema)
│   │       ├── parsers/
│   │       │   ├── mod.rs        # FormatParser trait
│   │       │   ├── common.rs     # Filesystem metadata (shared by all depths)
│   │       │   ├── pdf.rs        # [feature: pdf] lopdf + pdf-extract
│   │       │   ├── ooxml.rs      # [feature: ooxml] zip + quick-xml
│   │       │   ├── ole.rs        # [feature: ole] cfb
│   │       │   ├── plain_text.rs # Always included
│   │       │   ├── html.rs       # Always included
│   │       │   ├── rtf.rs        # Always included
│   │       │   ├── opendocument.rs
│   │       │   ├── email.rs      # [feature: email] mailparse
│   │       │   ├── image.rs      # [feature: image-meta] kamadak-exif + imagesize
│   │       │   └── archive.rs    # Always included
│   │       └── analysis/
│   │           ├── mod.rs
│   │           ├── xref.rs       # Cross-reference extraction (regex patterns)
│   │           ├── version.rs    # Filename version detection
│   │           ├── language.rs   # [feature: lang-detect] whichlang
│   │           ├── jargon.rs     # TF-IDF jargon detection
│   │           ├── department.rs # Directory → department inference
│   │           └── summary.rs    # Corpus-level aggregation
│   ├── muninn-cli/               # Binary: CLI entry point
│   │   └── src/
│   │       └── main.rs           # clap arg parsing → ScanConfig → Scanner::run
│   └── muninn-gui/               # Binary: GUI entry point (future)
├── resources/                    # Baked-in data (language model, background corpus)
└── tests/fixtures/               # Integration test data
```

## Quick Start

```sh
cargo run -p muninn-cli -- /path/to/scan/root
```

## Features

Format parsers are feature-gated. Disable unused formats to cut compile time and binary size:

```sh
cargo build -p muninn-cli --no-default-features --features pdf,ooxml
```

| Feature       | Crates pulled in              | Formats                        |
|---------------|-------------------------------|--------------------------------|
| `pdf`         | `lopdf`, `pdf-extract`        | `.pdf`                         |
| `ooxml`       | `zip`, `quick-xml`            | `.docx`, `.xlsx`, `.pptx` etc  |
| `ole`         | `cfb`                         | `.doc`, `.xls`, `.ppt`, `.msg` |
| `email`       | `mailparse`                   | `.eml`                         |
| `image-meta`  | `kamadak-exif`, `imagesize`   | JPEG, PNG, TIFF, etc           |
| `text-analysis` | `aho-corasick`              | Fast multi-pattern matching    |
| `lang-detect` | `whichlang`                   | Language classification        |

## Muninn: Sprint Plan

---

### Sprint 1 — Parsers

The core value proposition. Every parser is currently a stub that returns empty results. Each card below is one parser implementation.

---

#### 1.1 PDF Parser

**File:** `crates/muninn-core/src/parsers/pdf.rs`
**Depends on:** `lopdf`, `pdf-extract`

Highest-value parser — PDFs dominate most corporate corpora.

- [ ] Load PDF with `lopdf::Document::load`, extract `/Info` dictionary (Author, Title, Subject, Keywords, Producer, Creator)
- [ ] Parse `CreationDate` and `ModDate` from `/Info` into `doc.dates.document_created` / `document_modified` (handle PDF date format `D:YYYYMMDDHHmmSS`)
- [ ] Count pages via `document.get_pages().len()`, write to `doc.structure.page_count`
- [ ] Detect AcroForm presence (`document.get_object` on `/AcroForm`), write to `doc.pdf_specific.has_forms`
- [ ] Count embedded images by walking page resources for `/XObject` `/Subtype /Image` entries
- [ ] Per-page text extraction using `pdf-extract`, limited to first `text_extraction_depth` pages
- [ ] Classify `text_extractability`: count pages yielding >10 chars as `pages_with_text`, remainder as `pages_without_text`, derive `born_digital` / `scanned` / `mixed`
- [ ] Feed extracted text to `analysis::language::detect_language`, write `doc.language` / `doc.language_confidence`
- [ ] Feed extracted text to `analysis::xref::XrefExtractor::extract`, write `doc.cross_references`
- [ ] Wrap all `lopdf` / `pdf-extract` failures as `ExtractionError` with appropriate `stage` strings — never panic

---

#### 1.2 OOXML Parser (DOCX / XLSX / PPTX)

**File:** `crates/muninn-core/src/parsers/ooxml.rs`
**Depends on:** `zip`, `quick-xml`

One parser that branches by sub-format internally using the file extension.

- [ ] Open file with `zip::ZipArchive::new`, handle corrupt-zip errors gracefully
- [ ] Parse `docProps/core.xml` with `quick-xml`: extract `dc:creator`, `cp:lastModifiedBy`, `dcterms:created`, `dcterms:modified`, `dc:title`, `dc:subject`, `cp:keywords`, `cp:revision`
- [ ] Write authorship and dates to `doc.authorship` and `doc.dates.document_created` / `document_modified`
- [ ] Branch on extension for structural extraction:
  - [ ] **DOCX:** parse `word/document.xml` — count headings (and track max depth), tables, images (`<wp:inline>` / `<wp:anchor>`), footnotes/endnotes (from `word/footnotes.xml` / `word/endnotes.xml`), lists (detect `<w:numPr>`), comments (from `word/comments.xml`)
  - [ ] **XLSX:** count sheets from `xl/workbook.xml`, extract named ranges, compute row/column dimensions per sheet from `xl/worksheets/sheet*.xml`
  - [ ] **PPTX:** count slides from `ppt/presentation.xml`, count notes slides, count embedded images across slides
- [ ] Text extraction from content XML (first `text_extraction_depth` pages/slides/sheets), feed to language detection and xref extraction
- [ ] Handle macro-enabled variants (`.docm`, `.xlsm`, `.pptm`) — same logic, just different extensions

---

#### 1.3 OLE Parser (Legacy DOC / XLS / PPT / MSG)

**File:** `crates/muninn-core/src/parsers/ole.rs`
**Depends on:** `cfb`

- [ ] Open with `cfb::CompoundFile::open`, handle corrupt-file errors
- [ ] Read `\x05SummaryInformation` stream, parse the OLE property set format to extract: Title, Subject, Author, LastSavedBy, CreateTime, LastSaveTime
- [ ] Write extracted properties to `doc.authorship` and `doc.dates`
- [ ] For `.doc`: extract page count from `\x05DocumentSummaryInformation` if present
- [ ] For `.xls`: extract sheet count from Workbook stream (BIFF record parsing — keep minimal, just count `BoundSheet8` records)
- [ ] For `.ppt`: extract slide count from `PowerPoint Document` stream if feasible, otherwise leave null
- [ ] For `.msg`: parse Outlook MSG internal streams — extract sender (`PidTagSenderName`), recipients, subject (`PidTagSubject`), date (`PidTagMessageDeliveryTime`), attachment list (walk `__attach_version` substorages for filenames and sizes)
- [ ] Text extraction from `.doc` OLE stream for language detection (best-effort, this is notoriously unreliable)

---

#### 1.4 Plain Text Parser

**File:** `crates/muninn-core/src/parsers/plain_text.rs`
**Depends on:** `chardetng`, `encoding_rs`

- [ ] Read first 8192 bytes of the file, detect encoding with `chardetng::EncodingDetector`
- [ ] Write `doc.encoding` and `doc.encoding_confidence`
- [ ] Decode full file content using `encoding_rs` based on detected encoding
- [ ] Count lines and characters, store in `doc.structure` (repurpose `page_count` as line count, or add plain-text-specific fields — decide and document)
- [ ] For `.csv` / `.tsv`: count rows and columns (first row = column count), write to structure
- [ ] For `.json` / `.jsonl`: detect top-level keys or array length
- [ ] For `.xml`: extract root element name
- [ ] For `.yaml` / `.yml`: extract top-level keys
- [ ] Feed decoded text to language detection and xref extraction

---

#### 1.5 HTML Parser

**File:** `crates/muninn-core/src/parsers/html.rs`

- [ ] Add `scraper` (or `lol_html`) to workspace dependencies and `muninn-core` deps — this crate is currently missing
- [ ] Parse HTML, extract `<meta>` tags: `author`, `description`, `keywords`, `generator`
- [ ] Extract `<title>` text
- [ ] Count structural elements: headings (`h1`–`h6`), tables, images (`<img>`), links (`<a href>`)
- [ ] Write counts to `doc.structure`
- [ ] Extract body text (strip tags), feed to language detection and xref extraction

---

#### 1.6 Email Parser (.eml)

**File:** `crates/muninn-core/src/parsers/email.rs`
**Depends on:** `mailparse`

- [ ] Read file bytes, parse with `mailparse::parse_mail`
- [ ] Extract headers: `From`, `To`, `Cc`, `Date`, `Subject`
- [ ] Write sender to `doc.authorship.author`, date to `doc.dates.document_created`
- [ ] Catalog attachments: walk MIME parts, collect filenames and sizes for attachment parts
- [ ] Extract body text (prefer `text/plain` part, fall back to stripping HTML from `text/html`), feed to language detection

---

#### 1.7 RTF Parser

**File:** `crates/muninn-core/src/parsers/rtf.rs`

- [ ] Evaluate and add an RTF parsing crate (e.g. `rtf-parser`) to workspace deps — currently missing
- [ ] Parse `\info` group: extract `\author`, `\title`, `\creatim` fields
- [ ] Write to `doc.authorship` and `doc.dates`
- [ ] Strip RTF control words to extract plain text
- [ ] Feed text to language detection and xref extraction

---

#### 1.8 OpenDocument Parser (ODT / ODS / ODP)

**File:** `crates/muninn-core/src/parsers/opendocument.rs`
**Depends on:** `zip`, `quick-xml` (shared with OOXML)

- [ ] Open as zip, parse `meta.xml` for: author, creation date, modification date, editing cycles, generator application
- [ ] Write to `doc.authorship` and `doc.dates`
- [ ] For `.odt`: extract heading count, table count, page count from `<meta:document-statistic>` in `meta.xml`
- [ ] For `.ods`: count sheets from `content.xml`
- [ ] For `.odp`: count slides (draw:page elements) from `content.xml`
- [ ] Extract text from `content.xml`, feed to language detection and xref extraction

---

#### 1.9 Image Metadata Parser

**File:** `crates/muninn-core/src/parsers/image.rs`
**Depends on:** `kamadak-exif`, `imagesize`

- [ ] Read dimensions with `imagesize::size`, store in structure (repurpose or extend fields)
- [ ] For JPEG/TIFF: extract EXIF with `kamadak-exif` — camera info, GPS coordinates, date taken, software
- [ ] Write EXIF date to `doc.dates.document_created` if present
- [ ] For `.svg`: parse XML, extract `<title>`, `<metadata>`, and any embedded text content
- [ ] Handle missing/corrupt EXIF gracefully (most PNGs and BMPs won't have any)

---

#### 1.10 Archive / Container Parser

**File:** `crates/muninn-core/src/parsers/archive.rs`
**Depends on:** `zip` (shared with OOXML)

- [ ] For `.zip`: open with `zip::ZipArchive`, iterate entries to collect filenames, sizes, and total count
- [ ] Detect "actually a document" cases — if archive contains `[Content_Types].xml`, it's OOXML, flag it
- [ ] Compute contained file format distribution (extension → count map), store as a domain term or custom field
- [ ] For `.mbox`: parse as text, count `From ` header lines (message separators), extract date range from first/last `Date:` headers
- [ ] For `.tar.gz` / `.tar.bz2` / `.7z` / `.rar`: note these need additional crates or are deferred — log a warning and populate what's possible (file size, MIME type only)

---

### Sprint 2 — Analysis Passes

Wire the analysis modules into the scanner pipeline and implement the actual logic. These all depend on parsers producing extracted text.

---

#### 2.1 Cross-Reference Extraction

**File:** `crates/muninn-core/src/analysis/xref.rs`

- [ ] Implement `XrefExtractor::extract`: run `RegexSet::matches` against input text, collect match positions
- [ ] For each match, use individual `Regex` captures to extract the actual matched string (RegexSet only tells you *which* patterns matched, not *where*)
- [ ] Implement `classify_reference`: map pattern index to `ReferenceType` — ISO/ANSI/ASTM/CFR → `Standard`, user-provided patterns → `InternalId`, URL-like matches → `External`, fallback → `Unknown`
- [ ] Deduplicate references (same `reference_text` appearing multiple times in one document)
- [ ] Wire into `scanner.rs::process_file` — call after parser runs, only if parser produced extracted text

---

#### 2.2 Language Detection

**File:** `crates/muninn-core/src/analysis/language.rs`

- [ ] Implement `detect_language` using `whichlang::detect_language` — map its `Lang` enum to ISO 639-1 codes
- [ ] Handle edge cases: very short text (< 50 chars) should return `None` rather than a low-confidence guess
- [ ] Decide on confidence scoring — `whichlang` doesn't provide one natively; either return a fixed 1.0 when detected or use text length as a proxy, and document the choice
- [ ] Wire into `scanner.rs::process_file` — call with extracted text, write to `doc.language` / `doc.language_confidence`

---

#### 2.3 Jargon Detection (TF-IDF)

**File:** `crates/muninn-core/src/analysis/jargon.rs`

- [ ] Source a background English word frequency list (top 50k words from Wikipedia or Common Crawl dump), compress, add to `resources/`
- [ ] Embed the compressed corpus into the binary using `include_bytes!`, decompress at first use (lazy_static or `std::sync::OnceLock`)
- [ ] Implement tokenization: lowercase, split on whitespace/punctuation, filter stopwords
- [ ] Compute per-document term frequencies
- [ ] Compute TF-IDF scores against background corpus
- [ ] Return top N terms (configurable, default ~20) above a threshold score
- [ ] Wire into `scanner.rs::process_file`, write to `doc.domain_terms`

---

#### 2.4 Department Inference

**File:** `crates/muninn-core/src/analysis/department.rs`

- [ ] Implement a normalization map for common directory name abbreviations: `eng` → `Engineering`, `hr` → `Human Resources`, `legal` → `Legal`, `compliance` → `Compliance`, `admin` → `Administration`, `it` → `Information Technology`, `finance` → `Finance`, `ops` → `Operations`
- [ ] Make override matching use glob patterns instead of naive `contains` — use `globset` for consistency with exclusion patterns
- [ ] Add unit tests with realistic directory structures

---

#### 2.5 Version Detection Hardening

**File:** `crates/muninn-core/src/analysis/version.rs`

- [ ] Pre-compile regexes once (currently compiles on every call via `Regex::new` inside the loop) — use `std::sync::LazyLock` or a `once_cell::sync::Lazy`
- [ ] Add unit tests for every pattern from the spec: `_v1`, `_rev03`, `_r2`, `(1)`, `_FINAL`, `_DRAFT`, `_APPROVED`, `_2024-01-15`, `_20240115`
- [ ] Test edge cases: filenames with multiple version-like patterns, filenames that are *only* a version string, filenames with no stem after stripping

---

#### 2.6 Summary Aggregation

**File:** `crates/muninn-core/src/analysis/summary.rs`

- [ ] Add encoding distribution aggregation (already coded but verify it works end-to-end)
- [ ] Verify date distribution buckets against the schema — currently uses `signed_duration_since` which could behave oddly with future-dated files
- [ ] Add integration test: build a `Vec<DocumentMetadata>` with known values, assert `compute_summary` produces expected counts

---

### Sprint 3 — Pipeline & Infrastructure Fixes

Bugs, missing dependencies, and correctness issues that will block compilation or produce wrong results.

---

#### 3.1 Missing `hostname` Dependency

**File:** `crates/muninn-core/Cargo.toml`, `crates/muninn-core/src/scanner.rs`

`scanner.rs` calls `hostname::get()` but the `hostname` crate isn't in the dependency list. Won't compile.

- [ ] Add `hostname = "0.4"` to `[workspace.dependencies]` and `muninn-core` deps
- [ ] Verify `get_hostname()` works on Windows, macOS, and Linux

---

#### 3.2 Missing `dirs` Dependency

**File:** `crates/muninn-core/src/config.rs`

`ScanConfig::default()` falls back to `cwd` instead of the user's Desktop.

- [ ] Add `dirs = "5"` to workspace deps and `muninn-core` deps
- [ ] Replace `dirs_default_desktop()` with `dirs::desktop_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))`

---

#### 3.3 Missing HTML Parsing Dependency

**File:** `Cargo.toml` (workspace), `crates/muninn-core/Cargo.toml`

No HTML parser crate is listed. The HTML parser module can't do anything without one.

- [ ] Evaluate `scraper` vs `lol_html` vs `html5ever` — `scraper` is simplest for read-only extraction
- [ ] Add chosen crate to workspace deps and feature-gate it if desired

---

#### 3.4 Missing RTF Parsing Dependency

**File:** `Cargo.toml` (workspace), `crates/muninn-core/Cargo.toml`

Same issue — no RTF crate.

- [ ] Evaluate `rtf-parser` or `rtf-grimoire` for `\info` group extraction and control-word stripping
- [ ] Add to workspace deps

---

#### 3.5 Fix `common.rs` Full-File Read for MIME Detection

**File:** `crates/muninn-core/src/parsers/common.rs`

`detect_mime` calls `fs::read(path)` which loads the entire file into memory, then takes 8192 bytes. A 2GB video file would blow up memory.

- [ ] Replace with `File::open` + `BufReader` + `read_exact` (or `read` into an 8192-byte buffer)
- [ ] Handle files shorter than 8192 bytes

---

#### 3.6 Implement Filename Hashing

**File:** `crates/muninn-core/src/scanner.rs`

`config.include_full_paths` is read but never acted on. When `false`, filenames should be hashed while preserving directory structure.

- [ ] Add `sha2` (or use `std::hash`) to deps
- [ ] In `process_file`, after computing `relative_path`: if `!include_full_paths`, split path into directory + filename, hash the filename component, reassemble
- [ ] Add unit test verifying directory structure is preserved but filename is opaque

---

#### 3.7 Wire Remaining Analysis Passes into Scanner

**File:** `crates/muninn-core/src/scanner.rs`

`process_file` currently only calls version detection and department inference. The xref, language, and jargon passes are never invoked.

- [ ] Build `XrefExtractor` once in `Scanner::run` (compiling regex patterns), pass reference into `process_file`
- [ ] After parser runs, if extracted text is available, call `xref_extractor.extract(text)` → `doc.cross_references`
- [ ] Call `language::detect_language(text)` → `doc.language` / `doc.language_confidence`
- [ ] Call `jargon::extract_jargon_terms(text)` → `doc.domain_terms`
- [ ] This requires parsers to return extracted text — either add a return channel to `FormatParser::parse` or store extracted text in a new field on `DocumentMetadata` (transient, `#[serde(skip)]`)

---

#### 3.8 Richer OS Detection

**File:** `crates/muninn-core/src/scanner.rs`

Currently outputs `"linux x86_64"`. Spec expects `"Windows 11 Pro 23H2"`.

- [ ] Add `os_info = "3"` to workspace deps
- [ ] Replace `get_os()` with `os_info::get()` → format as `"{type} {version} {edition}"`

---

### Sprint 4 — Testing, Polish & GUI Stub

---

#### 4.1 Test Fixture Collection

**Directory:** `tests/fixtures/`

- [ ] Create or source a minimal sample file for each deep-inspection format: `.pdf`, `.docx`, `.xlsx`, `.pptx`, `.doc`, `.xls`, `.ppt`, `.odt`, `.ods`, `.odp`, `.rtf`, `.html`, `.eml`, `.msg`, `.csv`, `.json`, `.xml`, `.yaml`, `.md`, `.txt`
- [ ] Create sample files for medium-inspection formats: `.zip` (with known contents), `.png` (with EXIF), `.jpg` (with EXIF), `.svg` (with `<title>`), `.epub`
- [ ] Create a small directory tree (`tests/fixtures/sample-corpus/`) with realistic nesting, multiple departments, versioned filenames, and a mix of formats
- [ ] Ensure all fixture files are small (< 50KB each) and contain no real customer data

---

#### 4.2 Parser Unit Tests

- [ ] For each parser: write at least one test that opens the corresponding fixture file, calls `parse()`, and asserts non-null values in the expected `DocumentMetadata` fields
- [ ] PDF: assert page count, author, `text_extractability` classification
- [ ] OOXML: assert author, dates, heading count (DOCX), sheet count (XLSX), slide count (PPTX)
- [ ] OLE: assert author, title from summary properties
- [ ] Plain text: assert encoding detection, line count, CSV column count
- [ ] HTML: assert title, heading count, link count
- [ ] Email: assert sender, subject, date, attachment count
- [ ] Image: assert dimensions, EXIF date (for JPEG fixture)

---

#### 4.3 End-to-End Scanner Test

- [ ] Write an integration test that runs `Scanner::run` against `tests/fixtures/sample-corpus/`
- [ ] Assert `corpus_summary.total_documents` matches expected count
- [ ] Assert `format_distribution` contains expected extensions with correct counts
- [ ] Assert no documents have non-null `errors` (all fixtures should parse cleanly)
- [ ] Assert report serializes to valid JSON

---

#### 4.4 Output Schema Validation

- [ ] Add `jsonschema` to dev-dependencies
- [ ] Write a test that serializes a `ScanReport` to JSON, then validates it against `muninn-output-schema.json`
- [ ] Run this as part of the end-to-end test to catch any drift between Rust types and the JSON schema

---

#### 4.5 CLI Integration Tests

- [ ] Add `assert_cmd` and `predicates` to `muninn-cli` dev-deps (already in workspace)
- [ ] Test: `muninn --help` exits 0 and prints usage
- [ ] Test: `muninn /nonexistent/path` exits non-zero with "scan root not found" message
- [ ] Test: `muninn tests/fixtures/sample-corpus/` produces a valid JSON report file
- [ ] Test: `--hash-filenames` flag produces a report where filenames are opaque but directories are readable
- [ ] Test: `--no-hostname` flag produces a report with null `hostname` and `os`

---

#### 4.6 Progress Reporting

- [ ] Add a progress callback or channel to `Scanner` — emit file count, current file path, and percentage
- [ ] In the CLI, print a simple line-based progress indicator (e.g. `[1234/5678] Processing engineering/specs/...`) to stderr
- [ ] Respect `--quiet` flag (add to CLI) to suppress progress output
- [ ] This lays the groundwork for GUI progress bars later

---

#### 4.7 GUI Crate Stub

**Directory:** `crates/muninn-gui/`

- [ ] Uncomment `muninn-gui` in workspace `Cargo.toml`
- [ ] Create `Cargo.toml` with a dependency on `muninn-core` and a GUI framework (evaluate `iced`, `egui`, or `tauri`)
- [ ] Create minimal `src/main.rs` that opens a window with the three main-GUI elements from the input spec: scan root selector, output location, consent toggles
- [ ] Wire the "Scan" button to `Scanner::new(config).run()` on a background thread
- [ ] Display results or error when complete
