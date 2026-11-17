use crate::format::Format;
use anyhow::{anyhow, bail, Context, Result};
use image::ImageFormat;
use pdfium_render::prelude::*;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

pub fn convert(input: &Path, output: &Path, target: Format, dpi: u32, quality: u8) -> Result<()> {
    let pdfium = load_pdfium()?;
    let document = pdfium
        .load_pdf_from_file(input, None)
        .map_err(|err| anyhow!("failed to open PDF {}: {err}", input.display()))?;

    let page_count = document.pages().len();
    if page_count == 0 {
        bail!("{} has no pages", input.display());
    }

    let config = PdfRenderConfig::new().scale_page_by_factor(dpi as f32 / 72.0);

    for (index, page) in document.pages().iter().enumerate() {
        let image = page
            .render_with_config(&config)
            .map_err(|err| anyhow!("failed to render page {}: {err}", index + 1))?
            .as_image()
            .map_err(|err| anyhow!("failed to build image for page {}: {err}", index + 1))?;

        let path = page_output(output, target, index, page_count as usize);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        match target {
            Format::Png => image
                .save_with_format(&path, ImageFormat::Png)
                .with_context(|| format!("failed to write {}", path.display()))?,
            Format::Jpg => save_jpeg(&image.to_rgb8(), &path, quality)
                .with_context(|| format!("failed to write {}", path.display()))?,
            _ => unreachable!("pdf_raster only handles png and jpg"),
        }
    }

    Ok(())
}

fn page_output(output: &Path, target: Format, index: usize, page_count: usize) -> PathBuf {
    if page_count <= 1 {
        return output.to_path_buf();
    }
    let stem = output
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("page");
    output.with_file_name(format!("{stem}-{}.{}", index + 1, target.extension()))
}

fn save_jpeg(image: &image::RgbImage, path: &Path, quality: u8) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
    encoder.encode_image(image)?;
    Ok(())
}

fn load_pdfium() -> Result<Pdfium> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(value) = std::env::var_os("SCOUT_PDFIUM_PATH") {
        let path = PathBuf::from(value);
        if !path.as_os_str().is_empty() {
            add_candidate(&mut candidates, &path);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            add_candidate(&mut candidates, dir);
            add_candidate(
                &mut candidates,
                &dir.join("vendor").join("pdfium").join("x64"),
            );
        }
    }

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    add_candidate(
        &mut candidates,
        &manifest.join("vendor").join("pdfium").join("x64"),
    );

    for candidate in &candidates {
        if candidate.is_file() {
            if let Ok(bindings) = Pdfium::bind_to_library(candidate) {
                return Ok(Pdfium::new(bindings));
            }
        }
    }

    match Pdfium::bind_to_system_library() {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(_) => Err(anyhow!(
            "pdfium library not found; set SCOUT_PDFIUM_PATH or run scripts/fetch-pdfium.ps1"
        )),
    }
}

fn add_candidate(candidates: &mut Vec<PathBuf>, path: &Path) {
    if path.is_file() {
        candidates.push(path.to_path_buf());
    } else if path.is_dir() {
        candidates.push(Pdfium::pdfium_platform_library_name_at_path(path));
    }
}
