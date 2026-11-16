use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn bin() -> Command {
    Command::cargo_bin("scout-convert").expect("binary builds")
}

fn page_count(path: &Path) -> usize {
    lopdf::Document::load(path)
        .expect("valid pdf")
        .get_pages()
        .len()
}

fn write_png(path: &Path, color: [u8; 4]) {
    let image = image::RgbaImage::from_pixel(4, 4, image::Rgba(color));
    image.save(path).expect("png saved");
}

fn pdfium_available() -> bool {
    if let Some(value) = std::env::var_os("SCOUT_PDFIUM_PATH") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return true;
        }
        if path.is_dir() {
            return dir_has_pdfium(&path);
        }
    }
    let vendored = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vendor")
        .join("pdfium")
        .join("x64");
    dir_has_pdfium(&vendored)
}

fn dir_has_pdfium(dir: &Path) -> bool {
    fs::read_dir(dir)
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains("pdfium")
            })
        })
        .unwrap_or(false)
}

#[test]
fn text_to_pdf_and_back() {
    let dir = tempfile::tempdir().unwrap();
    let txt = dir.path().join("notes.txt");
    let pdf = dir.path().join("notes.pdf");
    let extracted = dir.path().join("extracted.txt");
    fs::write(&txt, "hello world\nsecond line\n").unwrap();

    bin().arg(&txt).arg("-o").arg(&pdf).assert().success();

    assert_eq!(page_count(&pdf), 1);
    let header = fs::read(&pdf).unwrap();
    assert!(header.starts_with(b"%PDF-"));

    bin()
        .arg(&pdf)
        .arg("-t")
        .arg("txt")
        .arg("-o")
        .arg(&extracted)
        .assert()
        .success();

    let text = fs::read_to_string(&extracted).unwrap();
    assert!(text.contains("hello world"), "got: {text:?}");
    assert!(text.contains("second line"), "got: {text:?}");
}

#[test]
fn markdown_defaults_to_sibling_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let md = dir.path().join("readme.md");
    fs::write(&md, "# Title\n\nSome **bold** text.\n").unwrap();

    bin().arg(&md).assert().success();

    let pdf = dir.path().join("readme.pdf");
    assert!(pdf.exists());
    assert_eq!(page_count(&pdf), 1);
}

#[test]
fn merges_multiple_images_into_one_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first.png");
    let second = dir.path().join("second.png");
    let out = dir.path().join("merged.pdf");
    write_png(&first, [255, 0, 0, 255]);
    write_png(&second, [0, 0, 255, 255]);

    bin()
        .arg(&first)
        .arg(&second)
        .arg("-o")
        .arg(&out)
        .assert()
        .success();

    assert_eq!(page_count(&out), 2);
}

#[test]
fn batch_directory_to_out_dir() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let out = dir.path().join("out");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("a.txt"), "alpha\n").unwrap();
    fs::write(source.join("b.csv"), "x,y\n1,2\n").unwrap();
    fs::write(source.join("ignored.bin"), "nope").unwrap();

    bin()
        .arg(&source)
        .arg("--out-dir")
        .arg(&out)
        .assert()
        .success();

    assert_eq!(page_count(&out.join("a.pdf")), 1);
    assert_eq!(page_count(&out.join("b.pdf")), 1);
    assert!(!out.join("ignored.pdf").exists());
}

#[test]
fn dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let txt = dir.path().join("notes.txt");
    fs::write(&txt, "text\n").unwrap();

    bin()
        .arg(&txt)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("notes.pdf"));

    assert!(!dir.path().join("notes.pdf").exists());
}

#[test]
fn overwrite_is_guarded() {
    let dir = tempfile::tempdir().unwrap();
    let txt = dir.path().join("notes.txt");
    let pdf = dir.path().join("notes.pdf");
    fs::write(&txt, "text\n").unwrap();
    fs::write(&pdf, "existing").unwrap();

    bin()
        .arg(&txt)
        .arg("-o")
        .arg(&pdf)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--overwrite"));

    bin()
        .arg(&txt)
        .arg("-o")
        .arg(&pdf)
        .arg("--overwrite")
        .assert()
        .success();

    assert_eq!(page_count(&pdf), 1);
}

#[test]
fn list_formats_exits_cleanly() {
    bin()
        .arg("--list-formats")
        .assert()
        .success()
        .stdout(predicate::str::contains("docx -> pdf"));
}

#[test]
fn missing_input_is_usage_error() {
    bin().assert().failure().code(2);
}

#[test]
fn unknown_input_format_errors() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("blob.xyz");
    fs::write(&file, "data").unwrap();

    bin()
        .arg(&file)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot determine format"));
}

#[test]
fn pdf_to_png_splits_pages() {
    if !pdfium_available() {
        eprintln!("skipping pdf_to_png_splits_pages: pdfium not available");
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let txt = dir.path().join("long.txt");
    let pdf = dir.path().join("long.pdf");
    let png = dir.path().join("page.png");

    let mut content = String::new();
    for index in 0..90 {
        content.push_str(&format!("line number {index}\n"));
    }
    fs::write(&txt, content).unwrap();

    bin().arg(&txt).arg("-o").arg(&pdf).assert().success();
    assert!(page_count(&pdf) >= 2);

    bin()
        .arg(&pdf)
        .arg("-t")
        .arg("png")
        .arg("--dpi")
        .arg("50")
        .arg("-o")
        .arg(&png)
        .assert()
        .success();

    assert!(dir.path().join("page-1.png").exists());
    assert!(dir.path().join("page-2.png").exists());
}
