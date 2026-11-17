use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::{Regex, RegexBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use termcolor::StandardStream;

use crate::output::{Match, Printer, SearchResult};

/// Options used to build a [`Matcher`].
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub pattern: Option<String>,
    pub regex: bool,
    pub ignore_case: bool,
}

pub struct Matcher {
    regex: Option<Regex>,
    literal: Option<String>,
}

impl Matcher {
    pub fn new(opts: &SearchOptions) -> Result<Self> {
        let raw_pattern = opts.pattern.clone().unwrap_or_default();
        if opts.regex || opts.ignore_case {
            let pattern = if opts.regex {
                raw_pattern
            } else {
                regex::escape(&raw_pattern)
            };
            let mut b = RegexBuilder::new(&pattern);
            b.case_insensitive(opts.ignore_case);
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

    #[inline]
    fn find_match(&self, line: &str) -> Option<(usize, usize)> {
        if let Some(re) = &self.regex {
            let m = re.find(line)?;
            return Some((m.start(), m.end() - m.start()));
        }
        let needle = self.literal.as_ref()?;
        if needle.is_empty() {
            return Some((0, 0));
        }
        let idx = line.find(needle)?;
        Some((idx, needle.len()))
    }
}

pub fn search_files(files: Vec<PathBuf>, matcher: &Matcher, printer: &Printer) -> Result<bool> {
    let need_details = printer.needs_details();
    let files_only = printer.files_with_matches();
    let results: Vec<SearchResult> = files
        .par_iter()
        .filter_map(
            |path| match search_file(path, matcher, need_details, files_only) {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("scout: {}: {err}", path.display());
                    None
                }
            },
        )
        .collect();

    let mut results = results;
    results.sort_by(|a, b| a.path.cmp(&b.path));

    let mut stdout = termcolor::StandardStream::stdout(printer.choice());
    let mut found = false;
    for res in &results {
        if res.count == 0 {
            continue;
        }
        found = true;
        printer.print_result(res, &mut stdout)?;
    }
    Ok(found)
}

fn search_file(
    path: &Path,
    matcher: &Matcher,
    need_details: bool,
    files_only: bool,
) -> Result<Option<SearchResult>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    search_reader(
        path,
        BufReader::with_capacity(64 * 1024, file),
        matcher,
        need_details,
        files_only,
    )
}

fn search_reader<R: BufRead>(
    path: &Path,
    reader: R,
    matcher: &Matcher,
    need_details: bool,
    files_only: bool,
) -> Result<Option<SearchResult>> {
    let mut reader = reader;
    {
        let probe = reader
            .fill_buf()
            .with_context(|| format!("failed to read {}", path.display()))?;
        if memchr::memchr(0, probe).is_some() {
            return Ok(None);
        }
    }
    let mut line_buf = Vec::new();
    let mut matches = Vec::new();
    let mut count = 0usize;
    let mut line_no = 0usize;

    loop {
        line_buf.clear();
        let n = reader
            .read_until(b'\n', &mut line_buf)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if n == 0 {
            break;
        }

        line_no += 1;
        if line_buf.ends_with(b"\n") {
            line_buf.pop();
            if line_buf.ends_with(b"\r") {
                line_buf.pop();
            }
        } else if line_buf.ends_with(b"\r") {
            line_buf.pop();
        }

        let line = String::from_utf8_lossy(&line_buf);

        if let Some((byte_offset, matched_len)) = matcher.find_match(&line) {
            count += 1;
            if need_details {
                let col = line[..byte_offset].chars().count() + 1;
                matches.push(Match {
                    line_no,
                    col,
                    byte_offset,
                    matched_len,
                    text: line.into_owned(),
                });
            } else if files_only {
                return Ok(Some(SearchResult {
                    path: path.to_path_buf(),
                    count: 1,
                    matches: Vec::new(),
                }));
            }
        }
    }

    if count == 0 {
        Ok(None)
    } else {
        Ok(Some(SearchResult {
            path: path.to_path_buf(),
            count,
            matches,
        }))
    }
}

/// Search data piped to stdin when `PATH` is `-`.
pub fn search_stdin(matcher: &Matcher, printer: &Printer) -> Result<bool> {
    let stdin = std::io::stdin();
    let result = search_reader(
        Path::new("<stdin>"),
        BufReader::new(stdin.lock()),
        matcher,
        printer.needs_details(),
        printer.files_with_matches(),
    )?;
    match result {
        Some(result) => {
            let mut stdout = StandardStream::stdout(printer.choice());
            printer.print_result(&result, &mut stdout)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn matcher(pattern: &str) -> Matcher {
        Matcher::new(&SearchOptions {
            pattern: Some(pattern.into()),
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn search_reader_uses_given_path_and_keeps_invalid_utf8_lines() {
        let result = search_reader(
            Path::new("<stdin>"),
            BufReader::new(Cursor::new(b"first\nna\xffme TODO here\n")),
            &matcher("TODO"),
            true,
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(result.path, Path::new("<stdin>"));
        assert_eq!(result.count, 1);
        assert_eq!(result.matches[0].line_no, 2);
    }
}
