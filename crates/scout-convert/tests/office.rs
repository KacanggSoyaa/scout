use assert_cmd::Command;
use std::fs;
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;

const PACKAGE_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

fn bin() -> Command {
    Command::cargo_bin("scout-convert").expect("binary builds")
}

fn write_docx(path: &Path, paragraphs: &[&str]) {
    let file = fs::File::create(path).expect("docx created");
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(CONTENT_TYPES.as_bytes()).unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(PACKAGE_RELS.as_bytes()).unwrap();

    let mut body = String::new();
    for paragraph in paragraphs {
        body.push_str(&format!("<w:p><w:r><w:t>{paragraph}</w:t></w:r></w:p>"));
    }
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
    );

    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.finish().unwrap();
}

fn page_count(path: &Path) -> usize {
    lopdf::Document::load(path)
        .expect("valid pdf")
        .get_pages()
        .len()
}

#[test]
fn docx_to_markdown_and_html() {
    let dir = tempfile::tempdir().unwrap();
    let docx = dir.path().join("sample.docx");
    write_docx(&docx, &["Hello from scout", "Second paragraph"]);

    let md = dir.path().join("sample.md");
    bin()
        .arg(&docx)
        .arg("-t")
        .arg("md")
        .arg("-o")
        .arg(&md)
        .assert()
        .success();
    let markdown = fs::read_to_string(&md).unwrap();
    assert!(markdown.contains("Hello from scout"), "got: {markdown:?}");
    assert!(markdown.contains("Second paragraph"), "got: {markdown:?}");

    let html = dir.path().join("sample.html");
    bin()
        .arg(&docx)
        .arg("-t")
        .arg("html")
        .arg("-o")
        .arg(&html)
        .assert()
        .success();
    let markup = fs::read_to_string(&html).unwrap();
    assert!(markup.contains("Hello from scout"), "got: {markup:?}");
}

#[test]
fn docx_to_pdf_contains_text() {
    let dir = tempfile::tempdir().unwrap();
    let docx = dir.path().join("sample.docx");
    write_docx(&docx, &["Rendered paragraph"]);
    let pdf = dir.path().join("sample.pdf");

    bin()
        .arg(&docx)
        .arg("-t")
        .arg("pdf")
        .arg("-o")
        .arg(&pdf)
        .assert()
        .success();

    assert!(fs::read(&pdf).unwrap().starts_with(b"%PDF-"));
    assert_eq!(page_count(&pdf), 1);

    let txt = dir.path().join("sample.txt");
    bin()
        .arg(&pdf)
        .arg("-t")
        .arg("txt")
        .arg("-o")
        .arg(&txt)
        .assert()
        .success();
    let text = fs::read_to_string(&txt).unwrap();
    assert!(text.contains("Rendered paragraph"), "got: {text:?}");
}

#[test]
fn corrupt_docx_reports_failure() {
    let dir = tempfile::tempdir().unwrap();
    let docx = dir.path().join("broken.docx");
    fs::write(&docx, b"this is not a zip archive").unwrap();

    bin()
        .arg(&docx)
        .arg("-t")
        .arg("pdf")
        .arg("-o")
        .arg(dir.path().join("broken.pdf"))
        .assert()
        .failure()
        .code(2);
}
