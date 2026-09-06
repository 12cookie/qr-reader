use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use qr_reader::{Options, read_qr_from_pdf};

/// Detect and decode QR codes on every page of a PDF document.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Path to the input PDF.
    pdf: PathBuf,

    /// Rendering resolution in DPI (higher is more reliable but slower).
    #[arg(long, default_value_t = 200.0)]
    dpi: f32,

    /// Multiplier for the one-shot retry when a page yields nothing
    /// (set to 1.0 to disable).
    #[arg(long, default_value_t = 2.0)]
    retry_scale: f32,

    /// Explicit path to the PDFium library file or its directory.
    #[arg(long)]
    pdfium_path: Option<PathBuf>,

    /// Emit results as JSON.
    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(found_any) if found_any => ExitCode::SUCCESS,
        // Nothing failed, but no QR codes were found anywhere.
        Ok(_) => ExitCode::from(2),
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool> {
    let cli = Cli::parse();

    let opts = Options {
        dpi: cli.dpi,
        retry_scale: cli.retry_scale,
        pdfium_path: cli.pdfium_path,
    };

    let pages = read_qr_from_pdf(&cli.pdf, &opts)?;
    let found_any = pages.iter().any(|p| !p.codes.is_empty());

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&pages)?);
    } else {
        for page in &pages {
            if page.codes.is_empty() {
                println!("page {}: (no QR code found)", page.page);
            } else {
                for (i, code) in page.codes.iter().enumerate() {
                    println!("page {} [{}]: {}", page.page, i + 1, code.data);
                }
            }
        }
    }

    Ok(found_any)
}
