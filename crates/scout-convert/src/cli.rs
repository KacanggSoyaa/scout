use crate::format::Format;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Convert images and documents between formats.
#[derive(Parser, Debug)]
#[command(
    name = "scout-convert",
    version,
    about = "Convert images and documents to PDF and other formats",
    long_about = "Convert images, text, Markdown, Office documents and PDFs. \
                  Native conversions run without external tools; LibreOffice, Pandoc \
                  and ImageMagick are used as fallbacks when available."
)]
pub struct Args {
    /// Input files or directories
    #[arg(value_name = "INPUT", required_unless_present = "list_formats")]
    pub inputs: Vec<PathBuf>,

    /// Target format (pdf, png, jpg, txt, html, md)
    #[arg(short = 't', long = "to", value_name = "FORMAT")]
    pub to: Option<Format>,

    /// Output file for a single input
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FILE",
        conflicts_with = "out_dir"
    )]
    pub output: Option<PathBuf>,

    /// Output directory (one output file per input)
    #[arg(long = "out-dir", value_name = "DIR")]
    pub out_dir: Option<PathBuf>,

    /// Merge multiple images into one PDF
    #[arg(long = "merge", conflicts_with = "no_merge")]
    pub merge: bool,

    /// Convert each image separately instead of merging
    #[arg(long = "no-merge")]
    pub no_merge: bool,

    /// Page size for PDF output
    #[arg(long = "page-size", value_name = "SIZE", default_value = "a4")]
    pub page_size: PageSize,

    /// Rasterization DPI for PDF to image conversion
    #[arg(long = "dpi", value_name = "N", default_value_t = 150)]
    pub dpi: u32,

    /// JPEG quality (1-100)
    #[arg(long = "quality", value_name = "N", default_value_t = 90, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub quality: u8,

    /// Conversion engine
    #[arg(long = "engine", value_name = "ENGINE", default_value = "auto")]
    pub engine: Engine,

    /// Overwrite existing output files
    #[arg(long = "overwrite")]
    pub overwrite: bool,

    /// Print planned conversions without writing files
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Number of parallel workers
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<usize>,

    /// List supported conversions and exit
    #[arg(long = "list-formats")]
    pub list_formats: bool,

    /// Print extra information
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Suppress per-file progress output
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    A3,
}

impl PageSize {
    /// Returns (width, height) in millimetres.
    pub fn mm(self) -> (f32, f32) {
        match self {
            PageSize::A4 => (210.0, 297.0),
            PageSize::Letter => (215.9, 279.4),
            PageSize::Legal => (215.9, 355.6),
            PageSize::A3 => (297.0, 420.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Engine {
    Auto,
    Native,
    Libreoffice,
    Pandoc,
    Magick,
}
