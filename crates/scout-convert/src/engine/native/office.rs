use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn convert(input: &Path, output: &Path) -> Result<()> {
    let result = office2pdf::convert(input)
        .map_err(|err| anyhow::anyhow!("failed to convert {}: {err:?}", input.display()))?;

    if !result.warnings.is_empty() {
        eprintln!(
            "scout-convert: {} warning(s) while converting {}",
            result.warnings.len(),
            input.display()
        );
    }

    fs::write(output, &result.pdf)
        .with_context(|| format!("failed to write {}", output.display()))?;
    Ok(())
}
