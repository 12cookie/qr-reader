# qr-reader

Detect and decode QR codes on every page of a PDF.

Each page is rasterized with [PDFium], the page image is scanned with
[`rqrr`], and the decoded payloads are printed (or emitted as JSON), grouped
by page.

## Requirements

The PDFium dynamic library must be available at runtime. Download a prebuilt
one from <https://github.com/bblanchon/pdfium-binaries> and either:

- put `libpdfium.{dylib,so,dll}` next to the `qr-reader` binary, or
- set `PDFIUM_DYNAMIC_LIB_PATH` to the directory containing it, or
- pass `--pdfium-path /path/to/lib`.

If a system-wide PDFium is installed it is used automatically.

## Usage

```
cargo run --release -- document.pdf
cargo run --release -- --json document.pdf
cargo run --release -- --dpi 300 scanned.pdf
```

Options:

| flag             | default | meaning                                                        |
|------------------|---------|----------------------------------------------------------------|
| `--dpi`          | 200     | render resolution; raise for small or low-quality codes        |
| `--retry-scale`  | 2.0     | if a page finds nothing, retry once at `dpi * this` (1 = off)  |
| `--pdfium-path`  | –       | explicit path to the PDFium library file or its directory      |
| `--json`         | off     | emit results as JSON                                           |

Exit codes: `0` at least one QR code found, `2` none found, `1` error.

## Library

```rust
use qr_reader::{read_qr_from_pdf, Options};

fn main() -> anyhow::Result<()> {
    let pages = read_qr_from_pdf("document.pdf", &Options::default())?;
    for page in pages {
        for code in page.codes {
            println!("page {}: {}", code.page, code.data);
        }
    }
    Ok(())
}
```

`QrCode` also carries the raw decoded `bytes` for payloads that are not valid
UTF-8.

[PDFium]: https://pdfium.googlesource.com/pdfium/
[`rqrr`]: https://crates.io/crates/rqrr
