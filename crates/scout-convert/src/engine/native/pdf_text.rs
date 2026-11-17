use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;

pub fn convert(input: &Path, output: &Path) -> Result<()> {
    let text = pdf_extract::extract_text(input)
        .map_err(|err| anyhow!("failed to extract text from {}: {err:?}", input.display()))?;
    fs::write(output, text).with_context(|| format!("failed to write {}", output.display()))?;
    Ok(())
}
