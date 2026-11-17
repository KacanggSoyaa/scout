use anyhow::{Context, Result};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use std::path::PathBuf;

use crate::cli::Args;

pub fn collect_files(args: &Args) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(&args.path);
    builder.hidden(!args.hidden);
    if args.no_ignore {
        builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false);
    }

    if !args.globs.is_empty() {
        let root = if args.path.is_dir() {
            args.path.clone()
        } else {
            args.path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        };
        let mut ob = OverrideBuilder::new(root);
        for g in &args.globs {
            ob.add(g).with_context(|| format!("invalid glob: {g}"))?;
        }
        builder.overrides(ob.build()?);
    }

    let mut files = Vec::new();
    if args.path.is_file() {
        files.push(args.path.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use clap::Parser;

    #[test]
    fn globs_limit_results_to_matching_files() {
        let dir = TempDir::new();
        dir.write("a.txt", b"TODO");
        dir.write("b.rs", b"TODO");
        let root = dir.path.to_str().unwrap();

        let args = Args::try_parse_from(["scout", "TODO", root, "-g", "*.txt"]).unwrap();
        let files = collect_files(&args).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.txt"));
    }

    #[test]
    fn hidden_files_need_the_hidden_flag() {
        let dir = TempDir::new();
        dir.write("a.txt", b"TODO");
        dir.write(".hidden.txt", b"TODO");
        let root = dir.path.to_str().unwrap();

        let args = Args::try_parse_from(["scout", "TODO", root]).unwrap();
        let files = collect_files(&args).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.txt"));

        let args = Args::try_parse_from(["scout", "TODO", root, "--hidden"]).unwrap();
        let files = collect_files(&args).unwrap();
        assert_eq!(files.len(), 2);
    }
}
