# CLAUDE.md — Operational Contract for Muninn

This file governs how you work on Muninn. The README describes what things are and why; this file tells you what you must do. Read both before making changes. If this file contradicts what you infer from the code, **this file wins** — flag the discrepancy to the user rather than silently following the code.

For architecture, module responsibilities, the pipeline diagram, the error model, and the parser contract, read the README. Do not duplicate that understanding here — reference it.

---

## Dynamic Refdoc Protocol

After completing any task that changes architecture, module boundaries, conventions, data flow, or the public API surface:

1. Review both this file and README.md for sections that are now stale.
2. Propose specific edits: "I changed X, which means section Y should be updated. Here's what I'd change: [diff]."
3. Make approved edits before closing the task.
4. If unsure whether a change warrants a doc update, ask.

---

## Quality Gates

Run `make all` before presenting changes. It runs every hard gate in order and fails fast:

```bash
make all   # fmt-check → lint → test → deny → machete
```

The individual targets are also available (`make fmt-check`, `make lint`, `make test`, `make deny`, `make machete`, `make audit`). For a quick local check when iterating, `make check` runs `cargo check --workspace --all-targets --all-features` to catch type errors without a full build. The pre-push git hook enforces fmt, clippy, and tests automatically, so passing `make all` locally means the push will succeed. CI enforces stricter gates than the local hooks — it additionally runs `cargo deny`, `cargo machete`, `cargo doc --no-deps` (with `-Dwarnings`), and minimal-features compilation checks.

---

## Reference Loading Rules

Reference files live in `docs/reference/`.

### Always load

Load these **before** making changes to Rust code. Read the relevant sections, then proceed.

- `docs/reference/rust-best-practices.md` — ownership, error handling, module organization, testing patterns, performance, anti-patterns. Consult before writing any non-trivial Rust.

### Load by trigger

| When you're about to... | Load from README |
|---|---|
| Add or modify a parser | README → Parser Architecture (the `FormatParser` contract) |
| Change `output.rs` types | README → Output Schema; also open `muninn-output-schema.json` and verify the types still match |
| Change `config.rs` | README → Input Model; also open the Input Specification |
| Add a file extension to `classify.rs` | README → Inspection Depth table; also open the Supported File Formats doc |
| Modify `scanner.rs` pipeline | README → Pipeline diagram |
| Change the public API of `muninn-core` | README → Module Responsibilities table |
| Add or change error types | README → Error Model (the `MuninnError` vs `ExtractionError` boundary) |
| Add a dependency | README → Tech Stack table; also open `muninn-crates.md` for rationale |

### Spec documents (source of truth)

When the code and a spec disagree, the spec wins. Flag the discrepancy to the user.

- **Muninn: Input Specification** — input tiers, default values, consent model
- **Muninn: Supported File Formats** — the full format registry, inspection depths, extraction details per format
- **muninn-output-schema.json** — JSON Schema for the report output
- **muninn-crates.md** — why each dependency exists

---

## Conventions

### Error Handling

`thiserror` for `muninn-core` library errors. `anyhow` in `muninn-cli` only. Never `anyhow` in library code — callers need to match on error variants.

Parsers are non-fatal. A single corrupt file must not abort the scan. Push an `ExtractionError` and continue. The boundary: broken file → `ExtractionError` (data). Broken scan root → `MuninnError` (control flow).

No `unwrap()` outside tests. `expect()` requires a message starting with `"contract violated: ..."` or `"hardcoded value: ..."` explaining why failure is impossible.

### Visibility

`pub(crate)` over `pub` within `muninn-core`. Only re-exports in `lib.rs` are `pub`. The public API surface is: `ScanConfig`, `Scanner`, `ScanReport`, `MuninnError`, `Result`.

### Feature Gating

When adding a new format parser, gate it behind a feature flag if it introduces a dependency heavier than ~50KB compiled. Follow the existing pattern: optional dependency in `Cargo.toml`, `#[cfg(feature = "...")]` on the module declaration in `parsers/mod.rs`, and a match arm with the same `#[cfg]` in `scanner.rs::get_parser()`.

### Match Exhaustiveness

Exhaustive `match` on `ParserKind` and `InspectionDepth` in dispatch logic. Never use `_` catchalls that would silently swallow a new variant. The `#[allow(unreachable_patterns)]` on the final `_ => None` in `get_parser()` exists only to catch feature-disabled variants — it is not a license to skip new variants.

### Output Schema Sync

`output.rs` types must match `muninn-output-schema.json` exactly. Every `Option` field uses `#[serde(skip_serializing_if = "Option::is_none")]`. When changing any type in `output.rs`, verify against the JSON schema. When the schema itself needs to change, update both the schema file and `output.rs` in the same commit.

### Tests

Unit tests go in `#[cfg(test)] mod tests` at the bottom of each file. Integration tests go in `tests/`. Test fixture files go in `tests/fixtures/` — keep them small (< 50KB each) and never include real customer data.

When fixing a bug, add a test that reproduces it before writing the fix.

### Naming

Follow Rust conventions: `snake_case` for functions/variables/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants. Parser structs are `{Format}Parser` (e.g. `PdfParser`, `OoxmlParser`). Error variants are `{Noun}` or `{Adjective}{Noun}` (e.g. `ScanRootNotFound`, `PermissionDenied`).

### Functions

Functions taking `&Vec<T>` or `&String` are a lint failure; use `&[T]` and `&str`. Prefer `impl AsRef<Path>` over `&Path` in public APIs that accept paths.

---

## Boundaries

**SAFE** — do freely:

- `cargo check` / `clippy` / `test` / `fmt`
- Implement TODO stubs inside existing parser and analysis modules
- Add unit tests and integration tests
- Add test fixture files to `tests/fixtures/`
- Create new submodules within existing module directories

**ASK FIRST** — get explicit confirmation:

- Add a dependency to any `Cargo.toml`
- Introduce `unsafe` anywhere
- Change MSRV or edition
- Modify `[profile.*]` or `[workspace.lints]`
- Change the `FormatParser` trait signature
- Modify `output.rs` types (must stay in sync with JSON schema)
- Change the public API surface of `muninn-core` (what `lib.rs` re-exports)
- Activate the `muninn-gui` workspace member
- Add a proc-macro crate

**NEVER:**

- Commit secrets, API keys, or customer data
- `cargo publish` — this is proprietary to Cake Intelligence
- Disable or weaken clippy lints in `Cargo.toml` without a written reason
- Use `#[allow(...)]` without an accompanying `// reason = "..."` comment
- Delete existing tests
- Use `unwrap()` outside of test code
- Let a parser `panic!` or return `Err` for a single-file failure — non-fatal errors go in `ExtractionError`
- Include real customer documents in test fixtures
- Use `anyhow` in `muninn-core`

---

## Known Defects and Missing Pieces

If your task touches any of these, flag it to the user rather than silently resolving or ignoring it.

- **`dirs` crate missing** — `config.rs` falls back to `cwd` instead of Desktop. TODO comment in code.
- **HTML parser crate missing** — no HTML parsing dependency exists. `html.rs` can't do anything without one.
- **RTF parser crate missing** — same situation as HTML.
- **`common.rs` reads entire file for MIME detection** — `fs::read(path)` loads the whole file into memory then takes 8192 bytes. Will blow up on large files.
- **Filename hashing not implemented** — `config.include_full_paths` is checked but never acted on in `scanner.rs`.
- **Analysis passes not wired** — `process_file()` only runs version detection and department inference. Cross-reference, language, and jargon passes are never called.
- **No extracted text pipeline** — parsers have no way to return extracted text to the scanner for downstream analysis passes. The `FormatParser` trait may need a return channel or a transient `#[serde(skip)]` field on `DocumentMetadata`.
- **OS detection is minimal** — outputs `"linux x86_64"` instead of `"Windows 11 Pro 23H2"` as the spec expects.
- **Zero tests exist** — no unit tests, no integration tests, no fixtures.
- **Complexity decision tree** — `complexity_tier` always outputs `null`. The model is not trained yet. This is by design for v1.

