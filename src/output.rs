use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::cli::Args;

#[derive(Debug, Clone)]
pub struct Match {
    pub line_no: usize,
    pub col: usize,
    pub byte_offset: usize,
    pub matched_len: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub count: usize,
    pub matches: Vec<Match>,
}

pub struct Printer {
    files_with_matches: bool,
    count: bool,
    no_color: bool,
}

impl Printer {
    pub fn new(args: &Args) -> Self {
        Self {
            files_with_matches: args.files_with_matches,
            count: args.count,
            no_color: args.no_color,
        }
    }

    pub fn choice(&self) -> ColorChoice {
        if self.no_color {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        }
    }

    pub fn needs_details(&self) -> bool {
        !self.files_with_matches && !self.count
    }

    pub fn files_with_matches(&self) -> bool {
        self.files_with_matches
    }

    pub fn print_result(&self, res: &SearchResult, stdout: &mut StandardStream) -> Result<()> {
        let path = res.path.display().to_string();

        if self.files_with_matches {
            set_color(stdout, Color::Magenta, true)?;
            writeln!(stdout, "{path}")?;
            stdout.reset()?;
            return Ok(());
        }

        if self.count {
            set_color(stdout, Color::Magenta, true)?;
            write!(stdout, "{path}")?;
            stdout.reset()?;
            write!(stdout, ":")?;
            set_color(stdout, Color::Green, true)?;
            writeln!(stdout, "{}", res.count)?;
            stdout.reset()?;
            return Ok(());
        }

        for m in &res.matches {
            set_color(stdout, Color::Magenta, true)?;
            write!(stdout, "{path}")?;
            stdout.reset()?;
            write!(stdout, ":")?;
            set_color(stdout, Color::Green, true)?;
            write!(stdout, "{}", m.line_no)?;
            stdout.reset()?;
            write!(stdout, ":")?;
            set_color(stdout, Color::Green, true)?;
            write!(stdout, "{}", m.col)?;
            stdout.reset()?;
            write!(stdout, ":")?;

            let end = m.byte_offset + m.matched_len;
            if end <= m.text.len() {
                let (pre, rest) = m.text.split_at(m.byte_offset);
                let (mid, post) = rest.split_at(m.matched_len);
                write!(stdout, "{pre}")?;
                set_color(stdout, Color::Red, true)?;
                write!(stdout, "{mid}")?;
                stdout.reset()?;
                writeln!(stdout, "{post}")?;
            } else {
                writeln!(stdout, "{}", m.text)?;
            }
        }
        Ok(())
    }
}

fn set_color(out: &mut StandardStream, color: Color, bold: bool) -> Result<()> {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color)).set_bold(bold);
    out.set_color(&spec)?;
    Ok(())
}

pub fn print_layout(no_color: bool) -> Result<()> {
    let choice = if no_color {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };
    let mut out = StandardStream::stdout(choice);

    let mut title = ColorSpec::new();
    title.set_fg(Some(Color::Cyan)).set_bold(true);
    out.set_color(&title)?;
    writeln!(&mut out, "scout — fast concurrent file search")?;
    out.reset()?;
    writeln!(&mut out, "-----------------------------------")?;
    writeln!(&mut out)?;

    section(&mut out, "Functions")?;
    item(&mut out, "Search", "Find literal text or regex matches in files")?;
    item(&mut out, "Case", "Toggle case-insensitive matching (-i)")?;
    item(&mut out, "Filter", "Include files by glob patterns (-g)")?;
    item(&mut out, "Paths", "List only matching file paths (-l)")?;
    item(&mut out, "Counts", "Show match counts per file (-c)")?;
    item(&mut out, "Workers", "Control parallel search jobs (-j N)")?;
    item(&mut out, "Scope", "Include hidden files (--hidden), disable ignores (--no-ignore)")?;
    item(&mut out, "Output", "Colored terminal results or plain text (--no-color)")?;
    writeln!(&mut out)?;

    section(&mut out, "Usage")?;
    writeln!(&mut out, "  scout \"pattern\" [PATH]")?;
    writeln!(&mut out, "  scout -i \"pattern\" .")?;
    writeln!(&mut out, "  scout -g \"*.rs\" \"fn main\" src")?;
    writeln!(&mut out, "  scout -e \"fn\\s+\\w+\" .")?;
    writeln!(&mut out, "  scout -l \"TODO\" .")?;
    writeln!(&mut out, "  scout -c \"TODO\" .")?;
    writeln!(&mut out)?;

    section(&mut out, "Options")?;
    writeln!(&mut out, "  -e, --regex                  Treat pattern as regex")?;
    writeln!(&mut out, "  -i, --ignore-case            Case-insensitive matching")?;
    writeln!(&mut out, "  -g, --glob <GLOB>            Include glob (repeatable)")?;
    writeln!(&mut out, "  -l, --files-with-matches     Print matching file paths only")?;
    writeln!(&mut out, "  -c, --count                  Print match counts per file")?;
    writeln!(&mut out, "  -j, --jobs <N>               Number of worker threads")?;
    writeln!(&mut out, "      --hidden                 Search hidden files")?;
    writeln!(&mut out, "      --no-ignore              Do not respect ignore files")?;
    writeln!(&mut out, "      --no-color               Disable colored output")?;
    writeln!(&mut out, "      --layout                 Show terminal layout of functions")?;
    writeln!(&mut out, "  -h, --help                   Show full help")?;
    writeln!(&mut out, "  -V, --version                Show version")?;
    out.reset()?;
    Ok(())
}

fn section(out: &mut StandardStream, name: &str) -> Result<()> {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(Color::Yellow)).set_bold(true);
    out.set_color(&spec)?;
    writeln!(out, "{name}")?;
    out.reset()?;
    Ok(())
}

fn item(out: &mut StandardStream, key: &str, desc: &str) -> Result<()> {
    let mut k = ColorSpec::new();
    k.set_fg(Some(Color::Green)).set_bold(true);
    out.set_color(&k)?;
    write!(out, "  {:<8}", key)?;
    out.reset()?;
    writeln!(out, " {desc}")?;
    Ok(())
}
