# AGENTS.md

Guidance for AI coding agents working in this repository.

## What this project is

`qr-reader` is a small Rust CLI + library that rasterises every page of a
PDF with [PDFium] (via the `pdfium-render` crate) and scans each rendered
page image for QR codes with `rqrr`. See `README.md` for usage.

- `src/lib.rs` — library API (`read_qr_from_pdf`, `scan_image`, `Options`).
- `src/main.rs` — CLI wrapper (`clap`-based argument parsing).

## Build, test, lint

```
cargo build --release
cargo test
cargo clippy --all-targets
cargo fmt
```

Run these before considering a change complete. There is currently no
automated test suite exercising PDF/QR behavior directly (no `tests/`
directory) — if you add non-trivial logic, add unit tests near it in
`src/lib.rs`.

## PDFium runtime dependency

PDFium itself is a native dynamic library, not a Rust crate — it is **not**
vendored or downloaded automatically. `cargo build` will succeed without it,
but running the binary requires `libpdfium.{so,dylib,dll}` to be resolvable
via one of (in order): `--pdfium-path`, `PDFIUM_DYNAMIC_LIB_PATH`, the
current directory, or the system library path. See `bind_pdfium` in
`src/lib.rs` and the README's "Requirements" section.

If you need to actually run the CLI (not just build it) while working in
this environment, download a prebuilt library from
<https://github.com/bblanchon/pdfium-binaries> matching the host platform
and point `PDFIUM_DYNAMIC_LIB_PATH` at it.

## `pdfium-render` version notes

This crate pins `pdfium-render = "0.9.4"`. When bumping this dependency:

- Check the crate's feature list (`cargo add pdfium-render --dry-run` or
  the crates.io page) before assuming a feature name still exists —
  the image-integration feature has been renamed across versions (e.g.
  a generic `image` feature became version-pinned features like
  `image_025` / `image_api` / `image_latest`). This repo currently uses
  `image_025` to match the `image = "0.25"` dependency in `Cargo.toml`;
  keep these two in sync.
- Check whether `PdfBitmap::as_image()` (used in `src/lib.rs`) still
  returns the same type — it has changed signature/return type between
  minor versions (e.g. it began returning a `Result` instead of a bare
  value), which will surface as a compile error at the call site in
  `render_and_scan`.
- After bumping, run `cargo build` and `cargo clippy` and fix any
  resulting type errors rather than reverting the version.

## Conventions

- Keep `Options` and its CLI equivalent in `main.rs` (`Cli` struct) in
  sync — every library option should be reachable from the CLI.
- Prefer `anyhow::Context` for error messages that name the file/page
  involved, matching the existing style.
- No emojis, no speculative abstractions — keep changes minimal and
  scoped to what's asked.
