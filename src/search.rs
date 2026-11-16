use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::{Regex, RegexBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use crate::cli::Args;
use crate::output::{Match, Printer, SearchResult};

pub struct Matcher {
    regex: Option<Regex>,
    literal: Option<String>,
}

impl Matcher {
    pub fn new(args: &Args) -> Result<Self> {
        let raw_pattern = args.pattern.clone().unwrap_or_default();
        if args.regex || args.ignore_case {
            let pattern = if args.regex {
                raw_pattern
            } else {
                regex::escape(&raw_pattern)
            };
            let mut b = RegexBuilder::new(&pattern);
            b.case_insensitive(args.ignore_case);
            let regex = b.build().context("invalid pattern")?;
            Ok(Self {
                regex: Some(regex),
                literal: None,
            })
        } else {
            Ok(Self {
                regex: None,
                literal: Some(raw_pattern),
            })
        }
    }

    fn find_match(&self, line: &str) -> Option<(usize, String)> {
        if let Some(re) = &self.regex {
            let m = re.find(line)?;
            let col = line[..m.start()].chars().count() + 1;
            return Some((col, m.as_str().to_string()));
        }
        let needle = self.literal.as_ref()?;
        if needle.is_empty() {
            return Some((1, String::new()));
        }
        let idx = line.find(needle)?;
        let col = line[..idx].chars().count() + 1;
        Some((col, needle.clone()))
    }
}

pub fn search_files(files: Vec<PathBuf>, matcher: &Matcher, printer: &Printer) -> Result<bool> {
    let results: Vec<SearchResult> = files
        .par_iter()
        .filter_map(|path| match search_file(path, matcher) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("scout: {}: {err}", path.display());
                None
            }
        })
        .collect();

    let mut results = results;
    results.sort_by(|a, b| a.path.cmp(&b.path));

    let mut found = false;
    for res in &results {
        if res.matches.is_empty() {
            continue;
        }
        found = true;
        printer.print_result(res)?;
    }
    Ok(found)
}

fn search_file(path: &Path, matcher: &Matcher) -> Result<Option<SearchResult>> {
    let mut file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

    let mut probe = [0u8; 8192];
    let n = file.read(&mut probe).unwrap_or(0);
    if probe[..n].contains(&0) {
        return Ok(None); // skip binary
    }

    // reopen to read from start
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut matches = Vec::new();
    for (idx, line_res) in reader.lines().enumerate() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => return Ok(None), // likely binary/invalid utf-8; skip
        };
        if let Some((col, matched)) = matcher.find_match(&line) {
            matches.push(Match {
                line_no: idx + 1,
                col,
                text: line,
                matched,
            });
        }
    }

    if matches.is_empty() {
        Ok(None)
    } else {
        Ok(Some(SearchResult {
            path: path.to_path_buf(),
            matches,
        }))
    }
}
