use crate::format::Format;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn convert(input: &Path, output: &Path, target: Format) -> Result<()> {
    let document = rdocx::Document::open(input)
        .with_context(|| format!("failed to open {}", input.display()))?;
    let content = match target {
        Format::Html => document.to_html(),
        Format::Md => document.to_markdown(),
        _ => unreachable!("docx_export only handles html and md"),
    };
    fs::write(output, content).with_context(|| format!("failed to write {}", output.display()))?;
    Ok(())
}

pub fn to_pdf(input: &Path, output: &Path) -> Result<()> {
    let document = rdocx::Document::open(input)
        .with_context(|| format!("failed to open {}", input.display()))?;
    document
        .save_pdf(output)
        .with_context(|| format!("failed to write {}", output.display()))?;
    Ok(())
}
