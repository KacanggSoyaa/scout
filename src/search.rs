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
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    search_reader(path, file, matcher)
}

fn search_reader<R: Read>(
    path: &Path,
    reader: R,
    matcher: &Matcher,
) -> Result<Option<SearchResult>> {
    let mut reader = BufReader::new(reader);
    if reader.fill_buf()?.contains(&0) {
        return Ok(None); // skip binary
    }

    let mut matches = Vec::new();
    let mut buf = Vec::new();
    let mut line_no = 0usize;
    loop {
        buf.clear();
        let n = reader
            .read_until(b'\n', &mut buf)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if n == 0 {
            break;
        }
        if buf.last() == Some(&b'\n') {
            buf.pop();
            if buf.last() == Some(&b'\r') {
                buf.pop();
            }
        }
        let line = String::from_utf8_lossy(&buf);
        if let Some((col, matched)) = matcher.find_match(&line) {
            matches.push(Match {
                line_no: line_no + 1,
                col,
                text: line.into_owned(),
                matched,
            });
        }
        line_no += 1;
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

/// Search data piped to stdin when `PATH` is `-`.
pub fn search_stdin(matcher: &Matcher, printer: &Printer) -> Result<bool> {
    let stdin = std::io::stdin();
    let result = search_reader(Path::new("<stdin>"), stdin.lock(), matcher)?;
    match result {
        Some(result) => {
            printer.print_result(&result)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use crate::test_support::TempDir;
    use clap::Parser;
    use std::io::Cursor;

    fn args(pattern: &str) -> Args {
        Args::try_parse_from(["scout", pattern]).unwrap()
    }

    #[test]
    fn literal_match_reports_char_column() {
        let matcher = Matcher::new(&args("TODO")).unwrap();
        assert_eq!(matcher.find_match("héllo TODO"), Some((7, "TODO".into())));
    }

    #[test]
    fn regex_ignore_case_keeps_matched_text() {
        let args = Args::try_parse_from(["scout", "-i", "-e", "todo"]).unwrap();
        let matcher = Matcher::new(&args).unwrap();
        assert_eq!(matcher.find_match("x ToDo y"), Some((3, "ToDo".into())));
    }

    #[test]
    fn invalid_utf8_does_not_drop_the_file() {
        let dir = TempDir::new();
        let path = dir.write("mixed.txt", b"first\nna\xffme TODO here\nlast\n");
        let result = search_file(&path, &Matcher::new(&args("TODO")).unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].line_no, 2);
        assert!(result.matches[0].text.contains("TODO"));
    }

    #[test]
    fn skips_files_with_nul_bytes() {
        let dir = TempDir::new();
        let path = dir.write("bin.dat", b"TODO\x00more");
        assert!(
            search_file(&path, &Matcher::new(&args("TODO")).unwrap())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn reader_path_is_used_for_stdin_results() {
        let matcher = Matcher::new(&args("TODO")).unwrap();
        let result = search_reader(Path::new("<stdin>"), Cursor::new(b"a\nTODO\n"), &matcher)
            .unwrap()
            .unwrap();
        assert_eq!(result.path, Path::new("<stdin>"));
        assert_eq!(result.matches[0].line_no, 2);
    }
}
