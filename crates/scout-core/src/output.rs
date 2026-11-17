use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

/// Output-related options derived from the CLI.
#[derive(Debug, Clone)]
pub struct PrintOptions {
    pub files_with_matches: bool,
    pub count: bool,
    pub no_color: bool,
}

pub struct Printer {
    files_with_matches: bool,
    count: bool,
    no_color: bool,
}

impl Printer {
    pub fn new(opts: &PrintOptions) -> Self {
        Self {
            files_with_matches: opts.files_with_matches,
            count: opts.count,
            no_color: opts.no_color,
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

pub fn set_color(out: &mut StandardStream, color: Color, bold: bool) -> Result<()> {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color)).set_bold(bold);
    out.set_color(&spec)?;
    Ok(())
}
