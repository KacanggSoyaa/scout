use crate::cli::PageSize;
use crate::format::Format;
use anyhow::{Context, Result};
use printpdf::{
    BuiltinFont, Color, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt,
    Rgb, TextItem,
};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::fs;
use std::path::Path;

const MARGIN_MM: f32 = 20.0;
const FONT_SIZE_PT: f32 = 10.0;
const LINE_HEIGHT_PT: f32 = 12.0;
const COURIER_CHAR_WIDTH: f32 = 0.6;

pub fn convert(input: &Path, output: &Path, page: PageSize, source: Format) -> Result<()> {
    let raw = fs::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    let decoded = String::from_utf8_lossy(&raw);
    let text = if source == Format::Md {
        markdown_to_text(&decoded)
    } else {
        decoded.into_owned()
    };

    let lines = layout_lines(&text, page);
    let (page_width, page_height) = page.mm();
    let line_height_mm = LINE_HEIGHT_PT * 25.4 / 72.0;
    let usable_height = page_height - MARGIN_MM * 2.0;
    let lines_per_page = ((usable_height / line_height_mm).floor() as usize).max(1);
    let start_y = page_height - MARGIN_MM;

    let mut document = PdfDocument::new("scout-convert");
    let mut pages = Vec::new();

    for chunk in lines.chunks(lines_per_page) {
        let mut ops = vec![
            Op::StartTextSection,
            Op::SetFont {
                font: PdfFontHandle::Builtin(BuiltinFont::Courier),
                size: Pt(FONT_SIZE_PT),
            },
            Op::SetFillColor {
                col: Color::Rgb(Rgb {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    icc_profile: None,
                }),
            },
        ];
        for (index, line) in chunk.iter().enumerate() {
            let y = start_y - index as f32 * line_height_mm;
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(MARGIN_MM), Mm(y)),
            });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(line.clone())],
            });
        }
        ops.push(Op::EndTextSection);
        pages.push(PdfPage::new(Mm(page_width), Mm(page_height), ops));
    }

    if pages.is_empty() {
        pages.push(PdfPage::new(Mm(page_width), Mm(page_height), Vec::new()));
    }

    let bytes = document
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    fs::write(output, bytes).with_context(|| format!("failed to write {}", output.display()))?;
    Ok(())
}

fn content_width_pt(page: PageSize) -> f32 {
    (page.mm().0 - MARGIN_MM * 2.0) * 72.0 / 25.4
}

fn max_chars(page: PageSize) -> usize {
    (content_width_pt(page) / (FONT_SIZE_PT * COURIER_CHAR_WIDTH))
        .floor()
        .max(1.0) as usize
}

fn layout_lines(text: &str, page: PageSize) -> Vec<String> {
    let max = max_chars(page);
    let mut lines = Vec::new();
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    for raw_line in normalized.split('\n') {
        let expanded = raw_line.replace('\t', "    ");
        let clean = sanitize(&expanded);
        if clean.is_empty() {
            lines.push(String::new());
            continue;
        }
        lines.extend(wrap_line(&clean, max));
    }
    lines
}

fn wrap_line(line: &str, max: usize) -> Vec<String> {
    if line.chars().count() <= max {
        return vec![line.to_string()];
    }

    let mut out = Vec::new();
    let mut rest = line;
    while rest.chars().count() > max {
        let byte_max = rest
            .char_indices()
            .nth(max)
            .map(|(index, _)| index)
            .unwrap_or(rest.len());
        let window = &rest[..byte_max];
        let split = window
            .rfind(' ')
            .filter(|&index| index > 0)
            .unwrap_or(window.len());
        out.push(rest[..split].trim_end().to_string());
        rest = rest[split..].trim_start();
        if rest.is_empty() {
            break;
        }
    }
    if !rest.is_empty() {
        out.push(rest.to_string());
    }
    out
}

/// Built-in PDF fonts use WinAnsi encoding, so keep only Latin-1 code points.
fn sanitize(line: &str) -> String {
    line.chars()
        .map(|ch| {
            if ch == ' '
                || (' '..='~').contains(&ch)
                || ((ch as u32) >= 0xA0 && (ch as u32) <= 0xFF)
            {
                ch
            } else {
                '?'
            }
        })
        .collect()
}

fn markdown_to_text(source: &str) -> String {
    let mut out = String::new();
    for event in Parser::new_ext(source, Options::empty()) {
        match event {
            Event::Text(text) | Event::Code(text) => out.push_str(&text),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::Start(Tag::Item) => out.push_str("- "),
            Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::CodeBlock) => {
                out.push('\n');
                out.push('\n');
            }
            Event::Rule => out.push_str("\n----\n"),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_at_spaces() {
        let wrapped = wrap_line("aaaa bbbb cccc", 8);
        assert_eq!(wrapped, vec!["aaaa", "bbbb", "cccc"]);
    }

    #[test]
    fn hard_wraps_long_words() {
        let wrapped = wrap_line("abcdefghij", 4);
        assert_eq!(wrapped, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn sanitizes_non_latin() {
        assert_eq!(sanitize("héllo 漢 ok"), "héllo ? ok");
    }

    #[test]
    fn markdown_becomes_plain_text() {
        let text = markdown_to_text("# Title\n\nSome **bold** text.");
        assert!(text.contains("Title"));
        assert!(text.contains("Some bold text."));
    }
}
