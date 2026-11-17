use super::ConvertOptions;
use crate::format::Format;
use crate::plan::Job;
use anyhow::{bail, Result};
use std::path::Path;

mod image_pdf;
mod pdf_text;
mod text_pdf;

#[cfg(feature = "docx-export")]
mod docx_export;
#[cfg(feature = "office")]
mod office;
#[cfg(feature = "pdf-raster")]
mod pdf_raster;

pub fn convert(job: &Job, opts: &ConvertOptions) -> Result<()> {
    let source = job.inputs[0].format;
    match (source, job.target) {
        (source, Format::Pdf) if source.is_image() => {
            image_pdf::convert(&job.inputs, &job.output, opts.page_size)
        }
        (source, Format::Pdf) if source.is_text() => {
            text_pdf::convert(&job.inputs[0].path, &job.output, opts.page_size, source)
        }
        (Format::Pdf, Format::Txt) => pdf_text::convert(&job.inputs[0].path, &job.output),
        (Format::Pdf, Format::Png) | (Format::Pdf, Format::Jpg) => {
            pdf_to_image(&job.inputs[0].path, &job.output, job.target, opts)
        }
        (Format::Docx, Format::Html) | (Format::Docx, Format::Md) => {
            docx_to_markup(&job.inputs[0].path, &job.output, job.target)
        }
        (Format::Docx, Format::Pdf) => docx_to_pdf(&job.inputs[0].path, &job.output),
        (Format::Xlsx, Format::Pdf) | (Format::Pptx, Format::Pdf) => {
            office_to_pdf(&job.inputs[0].path, &job.output)
        }
        _ => bail!(
            "native engine cannot convert {} -> {}; try --engine libreoffice or --engine pandoc",
            source.label(),
            job.target.label()
        ),
    }
}

#[cfg(feature = "pdf-raster")]
fn pdf_to_image(input: &Path, output: &Path, target: Format, opts: &ConvertOptions) -> Result<()> {
    pdf_raster::convert(input, output, target, opts.dpi, opts.quality)
}

#[cfg(not(feature = "pdf-raster"))]
fn pdf_to_image(_: &Path, _: &Path, _: Format, _: &ConvertOptions) -> Result<()> {
    bail!("PDF rasterization is disabled (enable the pdf-raster feature)")
}

#[cfg(feature = "docx-export")]
fn docx_to_markup(input: &Path, output: &Path, target: Format) -> Result<()> {
    docx_export::convert(input, output, target)
}

#[cfg(not(feature = "docx-export"))]
fn docx_to_markup(_: &Path, _: &Path, _: Format) -> Result<()> {
    bail!("DOCX export is disabled (enable the docx-export feature)")
}

#[cfg(feature = "docx-export")]
fn docx_to_pdf(input: &Path, output: &Path) -> Result<()> {
    docx_export::to_pdf(input, output)
}

#[cfg(all(not(feature = "docx-export"), feature = "office"))]
fn docx_to_pdf(input: &Path, output: &Path) -> Result<()> {
    office::convert(input, output)
}

#[cfg(all(not(feature = "docx-export"), not(feature = "office")))]
fn docx_to_pdf(_: &Path, _: &Path) -> Result<()> {
    bail!("DOCX to PDF is disabled (enable the docx-export or office feature)")
}

#[cfg(feature = "office")]
fn office_to_pdf(input: &Path, output: &Path) -> Result<()> {
    office::convert(input, output)
}

#[cfg(not(feature = "office"))]
fn office_to_pdf(_: &Path, _: &Path) -> Result<()> {
    bail!("Office conversion is disabled (enable the office feature)")
}
