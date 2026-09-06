//! Read QR codes out of every page of a PDF document.
//!
//! The PDF is rasterised page-by-page with PDFium, each page image is handed to
//! `rqrr` for QR detection/decoding, and the decoded payloads are returned
//! grouped by page.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::DynamicImage;
use pdfium_render::prelude::*;
use serde::Serialize;

/// One decoded QR code and where it was found.
#[derive(Debug, Clone, Serialize)]
pub struct QrCode {
    /// 1-based page number the code was found on.
    pub page: usize,
    /// Decoded payload, decoded as UTF-8 (lossy).
    pub data: String,
    /// Raw decoded bytes, in case the payload is not valid UTF-8.
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

/// Result for a single page.
#[derive(Debug, Clone, Serialize)]
pub struct PageResult {
    /// 1-based page number.
    pub page: usize,
    /// Every QR code decoded on this page (empty if none were found).
    pub codes: Vec<QrCode>,
}

/// Options controlling how the PDF is rendered before QR detection.
#[derive(Debug, Clone)]
pub struct Options {
    /// Rendering resolution. Higher = more reliable detection but slower.
    /// 200 is a good default for scanned documents.
    pub dpi: f32,
    /// If a page yields no QR codes, retry once at `dpi * retry_scale`.
    pub retry_scale: f32,
    /// Optional explicit path to the PDFium dynamic library (file or its
    /// containing directory). Falls back to `PDFIUM_DYNAMIC_LIB_PATH`, then the
    /// current directory, then the system library.
    pub pdfium_path: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dpi: 200.0,
            retry_scale: 2.0,
            pdfium_path: None,
        }
    }
}

/// Scan every page of `pdf_path` and return the QR codes found on each page.
pub fn read_qr_from_pdf(pdf_path: impl AsRef<Path>, opts: &Options) -> Result<Vec<PageResult>> {
    let pdf_path = pdf_path.as_ref();
    let pdfium = Pdfium::new(bind_pdfium(opts.pdfium_path.as_deref())?);

    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .with_context(|| format!("failed to open PDF: {}", pdf_path.display()))?;

    let mut results = Vec::new();

    for (index, page) in document.pages().iter().enumerate() {
        let page_number = index + 1;

        let mut codes = render_and_scan(&page, opts.dpi, page_number)
            .with_context(|| format!("failed to scan page {page_number}"))?;

        if codes.is_empty() && opts.retry_scale > 1.0 {
            codes = render_and_scan(&page, opts.dpi * opts.retry_scale, page_number)
                .with_context(|| format!("failed to re-scan page {page_number}"))?;
        }

        results.push(PageResult {
            page: page_number,
            codes,
        });
    }

    Ok(results)
}

/// Render one page at `dpi` and decode any QR codes in the resulting image.
fn render_and_scan(page: &PdfPage, dpi: f32, page_number: usize) -> Result<Vec<QrCode>> {
    // PDF user space is 72 units per inch.
    let scale = dpi / 72.0;

    let config = PdfRenderConfig::new().scale_page_by_factor(scale);

    let image = page
        .render_with_config(&config)
        .context("PDFium failed to render page")?
        .as_image()
        .context("failed to convert rendered page to an image")?;

    Ok(scan_image(&image, page_number))
}

/// Detect and decode every QR code in a single rendered image.
pub fn scan_image(image: &DynamicImage, page_number: usize) -> Vec<QrCode> {
    let luma = image.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);

    let mut out = Vec::new();
    for grid in prepared.detect_grids() {
        // `decode` fails on damaged / partially-detected codes; skip those.
        if let Ok((_meta, content)) = grid.decode() {
            out.push(QrCode {
                page: page_number,
                bytes: content.clone().into_bytes(),
                data: content,
            });
        }
    }
    out
}

/// Work out which PDFium library to bind to.
fn bind_pdfium(explicit: Option<&Path>) -> Result<Box<dyn PdfiumLibraryBindings>> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(p) = explicit {
        candidates.push(p.to_path_buf());
    }
    if let Ok(env_path) = std::env::var("PDFIUM_DYNAMIC_LIB_PATH") {
        candidates.push(PathBuf::from(env_path));
    }
    candidates.push(PathBuf::from("./"));

    for candidate in &candidates {
        // Accept either a directory or a direct path to the library file.
        let dir = if candidate.is_file() {
            candidate.parent().unwrap_or(Path::new("./"))
        } else {
            candidate.as_path()
        };
        let lib = Pdfium::pdfium_platform_library_name_at_path(dir);
        if let Ok(bindings) = Pdfium::bind_to_library(&lib) {
            return Ok(bindings);
        }
    }

    Pdfium::bind_to_system_library().context(
        "could not load the PDFium library. Download a build from \
         https://github.com/bblanchon/pdfium-binaries, then either put \
         libpdfium.dylib next to this binary or set PDFIUM_DYNAMIC_LIB_PATH \
         to its location",
    )
}
