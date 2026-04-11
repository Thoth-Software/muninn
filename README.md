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
