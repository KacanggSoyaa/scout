use anyhow::{Context, Result};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use std::path::PathBuf;

/// Options controlling which files are collected for searching.
#[derive(Debug, Clone)]
pub struct WalkOptions {
    pub path: PathBuf,
    pub globs: Vec<String>,
    pub hidden: bool,
    pub no_ignore: bool,
}

pub fn collect_files(opts: &WalkOptions) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(&opts.path);
    builder.hidden(!opts.hidden);
    if opts.no_ignore {
        builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false);
    }

    if !opts.globs.is_empty() {
        let root = if opts.path.is_dir() {
            opts.path.clone()
        } else {
            opts.path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        };
        let mut ob = OverrideBuilder::new(root);
        for g in &opts.globs {
            ob.add(g).with_context(|| format!("invalid glob: {g}"))?;
        }
        builder.overrides(ob.build()?);
    }

    let mut files = Vec::new();
    if opts.path.is_file() {
        files.push(opts.path.clone());
        return Ok(files);
    }

    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(err) => {
                eprintln!("scout: {err}");
                continue;
            }
        };
        let ft = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };
        if ft.is_file() {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}
