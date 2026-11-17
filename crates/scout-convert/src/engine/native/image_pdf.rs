use crate::cli::PageSize;
use crate::plan::InputFile;
use anyhow::{anyhow, Context, Result};
use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Pt, RawImage, XObjectTransform};
use std::fs;
use std::path::Path;

const MARGIN_MM: f32 = 10.0;

pub fn convert(inputs: &[InputFile], output: &Path, page: PageSize) -> Result<()> {
    let (page_width, page_height) = page.mm();
    let mut document = PdfDocument::new("scout-convert");
    let mut pages = Vec::with_capacity(inputs.len());

    for input in inputs {
        let bytes = fs::read(&input.path)
            .with_context(|| format!("failed to read {}", input.path.display()))?;
        let image = RawImage::decode_from_bytes(&bytes, &mut Vec::new())
            .map_err(|err| anyhow!("failed to decode {}: {err}", input.path.display()))?;

        let pixel_width = image.width.max(1) as f32;
        let pixel_height = image.height.max(1) as f32;
        let id = document.add_image(&image);

        let content_width = page_width - MARGIN_MM * 2.0;
        let content_height = page_height - MARGIN_MM * 2.0;
        let mut width = content_width;
        let mut height = width * pixel_height / pixel_width;
        if height > content_height {
            height = content_height;
            width = height * pixel_width / pixel_height;
        }
        let translate_x = MARGIN_MM + (content_width - width) / 2.0;
        let translate_y = MARGIN_MM + (content_height - height) / 2.0;

        let ops = vec![Op::UseXobject {
            id,
            transform: XObjectTransform {
                translate_x: Some(Pt(translate_x)),
                translate_y: Some(Pt(translate_y)),
                scale_x: Some(width / pixel_width),
                scale_y: Some(height / pixel_height),
                dpi: Some(72.0),
                ..Default::default()
            },
        }];
        pages.push(PdfPage::new(Mm(page_width), Mm(page_height), ops));
    }

    let bytes = document
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    fs::write(output, bytes).with_context(|| format!("failed to write {}", output.display()))?;
    Ok(())
}
