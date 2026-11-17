use crate::engine::ConvertOptions;
use crate::format::Format;
use crate::plan::Job;
use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn fallback(job: &Job, opts: &ConvertOptions) -> Result<()> {
    if job.inputs.len() != 1 {
        bail!("external fallbacks do not support merged inputs");
    }
    let source = job.inputs[0].format;

    if source.is_office()
        && matches!(job.target, Format::Pdf | Format::Html)
        && find_soffice().is_some()
    {
        return libreoffice(job, opts);
    }
    if matches!(
        (source, job.target),
        (Format::Docx, Format::Html)
            | (Format::Docx, Format::Md)
            | (Format::Md, Format::Html)
            | (Format::Txt, Format::Html)
    ) && find_pandoc().is_some()
    {
        return pandoc(job, opts);
    }
    if source.is_image() && job.target.is_image() && find_magick().is_some() {
        return magick(job, opts);
    }

    bail!(
        "no external fallback for {} -> {}",
        source.label(),
        job.target.label()
    )
}

pub fn libreoffice(job: &Job, opts: &ConvertOptions) -> Result<()> {
    let exe = find_soffice().ok_or_else(|| anyhow!("LibreOffice (soffice) not found"))?;
    if job.inputs.len() != 1 {
        bail!("LibreOffice engine does not support merged inputs");
    }
    let filter = match job.target {
        Format::Pdf => "pdf",
        Format::Html => "html",
        other => bail!("LibreOffice cannot convert to {}", other.label()),
    };

    let input = &job.inputs[0].path;
    let out_dir = job
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&out_dir)?;

    let status = Command::new(&exe)
        .arg("--headless")
        .arg("--norestore")
        .arg("--convert-to")
        .arg(filter)
        .arg("--outdir")
        .arg(&out_dir)
        .arg(input)
        .status()
        .with_context(|| format!("failed to run {}", exe.display()))?;
    if !status.success() {
        bail!("LibreOffice exited with {status}");
    }

    let produced = out_dir.join(output_name(input, job.target));
    if produced != job.output {
        if job.output.exists() && !opts.overwrite {
            bail!(
                "output already exists: {} (use --overwrite)",
                job.output.display()
            );
        }
        std::fs::rename(&produced, &job.output).or_else(|_| {
            std::fs::copy(&produced, &job.output)?;
            std::fs::remove_file(&produced)
        })?;
    }
    Ok(())
}

pub fn pandoc(job: &Job, _opts: &ConvertOptions) -> Result<()> {
    let exe = find_pandoc().ok_or_else(|| anyhow!("Pandoc not found"))?;
    if job.inputs.len() != 1 {
        bail!("Pandoc engine does not support merged inputs");
    }

    let status = Command::new(&exe)
        .arg(&job.inputs[0].path)
        .arg("-o")
        .arg(&job.output)
        .status()
        .with_context(|| format!("failed to run {}", exe.display()))?;
    if !status.success() {
        bail!("Pandoc exited with {status}");
    }
    Ok(())
}

pub fn magick(job: &Job, opts: &ConvertOptions) -> Result<()> {
    let exe = find_magick().ok_or_else(|| anyhow!("ImageMagick (magick) not found"))?;
    if job.inputs.len() != 1 {
        bail!("ImageMagick engine does not support merged inputs");
    }

    let mut command = Command::new(&exe);
    command.arg(&job.inputs[0].path);
    if job.target == Format::Jpg {
        command.arg("-quality").arg(opts.quality.to_string());
    }
    let status = command
        .arg(&job.output)
        .status()
        .with_context(|| format!("failed to run {}", exe.display()))?;
    if !status.success() {
        bail!("ImageMagick exited with {status}");
    }
    Ok(())
}

fn output_name(input: &Path, target: Format) -> String {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("output");
    format!("{stem}.{}", target.extension())
}

fn find_in_path(names: &[&str]) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn find_soffice() -> Option<PathBuf> {
    if let Some(path) = find_in_path(&["soffice.exe", "soffice"]) {
        return Some(path);
    }
    for base in [
        r"C:\Program Files\LibreOffice\program",
        r"C:\Program Files (x86)\LibreOffice\program",
    ] {
        let candidate = Path::new(base).join("soffice.exe");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn find_pandoc() -> Option<PathBuf> {
    if let Some(path) = find_in_path(&["pandoc.exe", "pandoc"]) {
        return Some(path);
    }
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA")?)
        .join("Pandoc")
        .join("pandoc.exe");
    local.is_file().then_some(local)
}

pub fn find_magick() -> Option<PathBuf> {
    find_in_path(&["magick.exe", "magick"])
}
