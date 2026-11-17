use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ValueEnum)]
pub enum Format {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Png,
    #[value(alias = "jpeg")]
    Jpg,
    Txt,
    Html,
    #[value(alias = "markdown")]
    Md,
    Csv,
    Webp,
    Bmp,
    #[value(alias = "tif")]
    Tiff,
    Gif,
    Json,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Xlsx => "xlsx",
            Format::Pptx => "pptx",
            Format::Png => "png",
            Format::Jpg => "jpg",
            Format::Txt => "txt",
            Format::Html => "html",
            Format::Md => "md",
            Format::Csv => "csv",
            Format::Webp => "webp",
            Format::Bmp => "bmp",
            Format::Tiff => "tiff",
            Format::Gif => "gif",
            Format::Json => "json",
        }
    }

    pub fn label(self) -> &'static str {
        self.extension()
    }

    pub fn is_image(self) -> bool {
        matches!(
            self,
            Format::Png | Format::Jpg | Format::Webp | Format::Bmp | Format::Tiff | Format::Gif
        )
    }

    pub fn is_text(self) -> bool {
        matches!(self, Format::Txt | Format::Csv | Format::Json | Format::Md)
    }

    pub fn is_office(self) -> bool {
        matches!(self, Format::Docx | Format::Xlsx | Format::Pptx)
    }

    pub fn is_output_supported(self) -> bool {
        matches!(
            self,
            Format::Pdf | Format::Png | Format::Jpg | Format::Txt | Format::Html | Format::Md
        )
    }

    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "pdf" => Some(Format::Pdf),
            "docx" => Some(Format::Docx),
            "xlsx" => Some(Format::Xlsx),
            "pptx" => Some(Format::Pptx),
            "png" => Some(Format::Png),
            "jpg" | "jpeg" => Some(Format::Jpg),
            "txt" => Some(Format::Txt),
            "html" | "htm" => Some(Format::Html),
            "md" | "markdown" => Some(Format::Md),
            "csv" => Some(Format::Csv),
            "webp" => Some(Format::Webp),
            "bmp" => Some(Format::Bmp),
            "tif" | "tiff" => Some(Format::Tiff),
            "gif" => Some(Format::Gif),
            "json" => Some(Format::Json),
            _ => None,
        }
    }

    /// Detect a format from the file extension, falling back to magic bytes.
    pub fn from_path(path: &Path) -> Result<Self> {
        if let Some(format) = Self::from_extension(path) {
            return Ok(format);
        }
        if let Some(format) = detect_magic(path)? {
            return Ok(format);
        }
        bail!(
            "cannot determine format of {}: use --to or a known extension",
            path.display()
        )
    }
}

fn detect_magic(path: &Path) -> Result<Option<Format>> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut buf = [0u8; 64 * 1024];
    let read = file
        .read(&mut buf)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let bytes = &buf[..read];

    if bytes.starts_with(b"%PDF-") {
        return Ok(Some(Format::Pdf));
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok(Some(Format::Png));
    }
    if bytes.starts_with(b"\xFF\xD8\xFF") {
        return Ok(Some(Format::Jpg));
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Ok(Some(Format::Gif));
    }
    if bytes.starts_with(b"BM") {
        return Ok(Some(Format::Bmp));
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(Some(Format::Webp));
    }
    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return Ok(Some(Format::Tiff));
    }
    if bytes.starts_with(b"PK\x03\x04") {
        let contains = |needle: &[u8]| bytes.windows(needle.len()).any(|window| window == needle);
        if contains(b"word/document.xml") || contains(b"word/") {
            return Ok(Some(Format::Docx));
        }
        if contains(b"xl/workbook.xml") || contains(b"xl/") {
            return Ok(Some(Format::Xlsx));
        }
        if contains(b"ppt/presentation.xml") || contains(b"ppt/") {
            return Ok(Some(Format::Pptx));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extension_mapping() {
        assert_eq!(
            Format::from_extension(Path::new("a.PDF")),
            Some(Format::Pdf)
        );
        assert_eq!(
            Format::from_extension(Path::new("a.jpeg")),
            Some(Format::Jpg)
        );
        assert_eq!(
            Format::from_extension(Path::new("a.markdown")),
            Some(Format::Md)
        );
        assert_eq!(
            Format::from_extension(Path::new("a.tif")),
            Some(Format::Tiff)
        );
        assert_eq!(Format::from_extension(Path::new("a.unknown")), None);
    }

    #[test]
    fn magic_pdf_and_png() {
        let dir = tempfile::tempdir().unwrap();
        let pdf = dir.path().join("noext");
        std::fs::write(&pdf, b"%PDF-1.7\n").unwrap();
        assert_eq!(Format::from_path(&pdf).unwrap(), Format::Pdf);

        let png = dir.path().join("image");
        let mut file = std::fs::File::create(&png).unwrap();
        file.write_all(b"\x89PNG\r\n\x1a\n").unwrap();
        assert_eq!(Format::from_path(&png).unwrap(), Format::Png);
    }

    #[test]
    fn magic_zip_sniff() {
        let dir = tempfile::tempdir().unwrap();
        let docx = dir.path().join("blob");
        std::fs::write(&docx, b"PK\x03\x04word/document.xml").unwrap();
        assert_eq!(Format::from_path(&docx).unwrap(), Format::Docx);
    }

    #[test]
    fn unknown_format_errors() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("blob");
        std::fs::write(&file, b"not a known format").unwrap();
        assert!(Format::from_path(&file).is_err());
    }
}
