use crate::cli::Args;
use crate::format::Format;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct InputFile {
    pub path: PathBuf,
    pub format: Format,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub inputs: Vec<InputFile>,
    pub output: PathBuf,
    pub target: Format,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub jobs: Vec<Job>,
}

pub fn build(args: &Args) -> Result<Plan> {
    let files = collect_inputs(&args.inputs)?;
    if files.is_empty() {
        bail!("no supported input files found");
    }

    let target = resolve_target(args, &files)?;
    if !target.is_output_supported() {
        bail!("cannot convert to {}", target.label());
    }

    let all_images = files.iter().all(|file| file.format.is_image());
    let merge = if args.merge {
        if !all_images {
            bail!("--merge requires image inputs only");
        }
        if target != Format::Pdf {
            bail!("--merge requires PDF output");
        }
        true
    } else {
        !args.no_merge
            && all_images
            && files.len() > 1
            && target == Format::Pdf
            && args.output.is_some()
    };

    if merge {
        let output = match &args.output {
            Some(output) => output.clone(),
            None => {
                let dir = args
                    .out_dir
                    .clone()
                    .or_else(|| files[0].path.parent().map(Path::to_path_buf))
                    .unwrap_or_else(|| PathBuf::from("."));
                dir.join("merged.pdf")
            }
        };
        ensure_distinct(&files, &output)?;
        return Ok(Plan {
            jobs: vec![Job {
                inputs: files,
                output,
                target: Format::Pdf,
            }],
        });
    }

    if args.output.is_some() && files.len() != 1 {
        bail!("-o/--output requires exactly one input; use --out-dir or --merge");
    }

    let mut jobs = Vec::with_capacity(files.len());
    for file in files {
        let output = output_for(args, &file, target);
        ensure_distinct(std::slice::from_ref(&file), &output)?;
        jobs.push(Job {
            inputs: vec![file],
            output,
            target,
        });
    }

    Ok(Plan { jobs })
}

fn collect_inputs(paths: &[PathBuf]) -> Result<Vec<InputFile>> {
    let mut files = Vec::new();
    let mut seen = HashSet::new();

    for input in paths {
        if input.is_dir() {
            for entry in WalkDir::new(input).follow_links(false) {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(_) => continue,
                };
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.into_path();
                if let Some(format) = Format::from_extension(&path) {
                    if seen.insert(path.clone()) {
                        files.push(InputFile { path, format });
                    }
                }
            }
        } else if input.is_file() {
            let format = Format::from_path(input)
                .with_context(|| format!("unsupported input {}", input.display()))?;
            if seen.insert(input.clone()) {
                files.push(InputFile {
                    path: input.clone(),
                    format,
                });
            }
        } else {
            bail!("input not found: {}", input.display());
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn resolve_target(args: &Args, files: &[InputFile]) -> Result<Format> {
    if let Some(target) = args.to {
        return Ok(target);
    }
    if let Some(output) = &args.output {
        if let Some(target) = Format::from_extension(output) {
            return Ok(target);
        }
        bail!("cannot infer target from output name; pass --to");
    }
    if files
        .iter()
        .all(|file| file.format.is_image() || file.format.is_office() || file.format.is_text())
    {
        return Ok(Format::Pdf);
    }
    bail!("specify --to or an output path with a known extension");
}

fn output_for(args: &Args, file: &InputFile, target: Format) -> PathBuf {
    if let Some(output) = &args.output {
        if output.is_dir() {
            return output.join(output_name(&file.path, target));
        }
        return output.clone();
    }
    if let Some(dir) = &args.out_dir {
        return dir.join(output_name(&file.path, target));
    }
    file.path.with_extension(target.extension())
}

fn output_name(path: &Path, target: Format) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("output");
    format!("{stem}.{}", target.extension())
}

fn ensure_distinct(inputs: &[InputFile], output: &Path) -> Result<()> {
    if inputs.iter().any(|input| input.path == output) {
        bail!(
            "refusing to overwrite input file with output: {}",
            output.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn args(items: &[&str]) -> Args {
        let mut argv = vec!["scout-convert"];
        argv.extend_from_slice(items);
        Args::parse_from(argv)
    }

    fn touch(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, b"data").unwrap();
        path
    }

    #[test]
    fn images_default_to_pdf_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let png = touch(dir.path(), "a.png");
        let plan = build(&args(&[png.to_str().unwrap()])).unwrap();
        assert_eq!(plan.jobs.len(), 1);
        assert_eq!(plan.jobs[0].target, Format::Pdf);
        assert_eq!(plan.jobs[0].output, dir.path().join("a.pdf"));
    }

    #[test]
    fn multiple_images_with_output_merge() {
        let dir = tempfile::tempdir().unwrap();
        let a = touch(dir.path(), "a.png");
        let b = touch(dir.path(), "b.png");
        let out = dir.path().join("book.pdf");
        let plan = build(&args(&[
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ]))
        .unwrap();
        assert_eq!(plan.jobs.len(), 1);
        assert_eq!(plan.jobs[0].inputs.len(), 2);
    }

    #[test]
    fn multiple_inputs_single_output_rejected_without_merge() {
        let dir = tempfile::tempdir().unwrap();
        let a = touch(dir.path(), "a.png");
        let b = touch(dir.path(), "b.png");
        let out = dir.path().join("out.pdf");
        let result = build(&args(&[
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--no-merge",
        ]));
        assert!(result.is_err());
    }

    #[test]
    fn out_dir_keeps_stems() {
        let dir = tempfile::tempdir().unwrap();
        let a = touch(dir.path(), "a.txt");
        let b = touch(dir.path(), "b.csv");
        let out_dir = dir.path().join("out");
        let plan = build(&args(&[
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ]))
        .unwrap();
        assert_eq!(plan.jobs.len(), 2);
        assert_eq!(plan.jobs[0].output, out_dir.join("a.pdf"));
        assert_eq!(plan.jobs[1].output, out_dir.join("b.pdf"));
    }

    #[test]
    fn refuses_to_overwrite_input() {
        let dir = tempfile::tempdir().unwrap();
        let pdf = touch(dir.path(), "a.pdf");
        let result = build(&args(&[
            pdf.to_str().unwrap(),
            "-t",
            "pdf",
            "-o",
            pdf.to_str().unwrap(),
        ]));
        assert!(result.is_err());
    }
}
