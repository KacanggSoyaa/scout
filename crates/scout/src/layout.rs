use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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
    item(
        &mut out,
        "Search",
        "Find literal text or regex matches in files",
    )?;
    item(&mut out, "Case", "Toggle case-insensitive matching (-i)")?;
    item(&mut out, "Filter", "Include files by glob patterns (-g)")?;
    item(&mut out, "Paths", "List only matching file paths (-l)")?;
    item(&mut out, "Counts", "Show match counts per file (-c)")?;
    item(&mut out, "Workers", "Control parallel search jobs (-j N)")?;
    item(
        &mut out,
        "Scope",
        "Include hidden files (--hidden), disable ignores (--no-ignore)",
    )?;
    item(
        &mut out,
        "Output",
        "Colored terminal results or plain text (--no-color)",
    )?;
    item(
        &mut out,
        "Convert",
        "Use scout-convert for PDF and document conversion",
    )?;
    writeln!(&mut out)?;

    section(&mut out, "Usage")?;
    writeln!(&mut out, "  scout \"pattern\" [PATH]")?;
    writeln!(&mut out, "  scout -i \"pattern\" .")?;
    writeln!(&mut out, "  scout -g \"*.rs\" \"fn main\" src")?;
    writeln!(&mut out, "  scout -e \"fn\\s+\\w+\" .")?;
    writeln!(&mut out, "  scout -l \"TODO\" .")?;
    writeln!(&mut out, "  scout -c \"TODO\" .")?;
    writeln!(&mut out, "  scout-convert photo.jpg -o photo.pdf")?;
    writeln!(&mut out)?;

    section(&mut out, "Options")?;
    writeln!(
        &mut out,
        "  -e, --regex                  Treat pattern as regex"
    )?;
    writeln!(
        &mut out,
        "  -i, --ignore-case            Case-insensitive matching"
    )?;
    writeln!(
        &mut out,
        "  -g, --glob <GLOB>            Include glob (repeatable)"
    )?;
    writeln!(
        &mut out,
        "  -l, --files-with-matches     Print matching file paths only"
    )?;
    writeln!(
        &mut out,
        "  -c, --count                  Print match counts per file"
    )?;
    writeln!(
        &mut out,
        "  -j, --jobs <N>               Number of worker threads"
    )?;
    writeln!(
        &mut out,
        "      --hidden                 Search hidden files"
    )?;
    writeln!(
        &mut out,
        "      --no-ignore              Do not respect ignore files"
    )?;
    writeln!(
        &mut out,
        "      --no-color               Disable colored output"
    )?;
    writeln!(
        &mut out,
        "      --layout                 Show terminal layout of functions"
    )?;
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
