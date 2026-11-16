# scout

A fast, concurrent CLI toolkit in Rust. The workspace ships two binaries:

- **`scout`** — a literal-first, grep/find-like file searcher.
- **`scout-convert`** — an image and document converter (images/text/Office to
  PDF, PDF to text/images, DOCX to HTML/Markdown).

Windows is the supported platform.

## Build

```bash
cargo build --release
```

Binaries are written to `target/release/scout.exe` and
`target/release/scout-convert.exe`.

> On Windows with the MSVC Rust toolchain, install **Visual Studio Build Tools**
> with “Desktop development with C++” (provides `link.exe`). Without a linker,
> `cargo build`/`cargo run` will fail.

Run without installing:

```bash
cargo run -p scout -- "pattern" .
cargo run -p scout-convert -- input.png -o output.pdf
```

## scout (search)

```bash
# Show terminal layout of available functions
scout
scout --layout

# Search
scout "pattern" [PATH]
scout -i "pattern" .
scout -g "*.rs" "fn main" src
scout -e "fn\s+\w+" .
scout -l "TODO" .
scout -c "TODO" .
```

### Options

- `pattern` optional search pattern (literal by default); omit to show the layout
- `PATH` file or directory to search (default: `.`)
- `-e, --regex` treat pattern as a regular expression
- `-i, --ignore-case` case-insensitive matching
- `-g, --glob <GLOB>` include files matching glob (repeatable)
- `-l, --files-with-matches` print only names of files with matches
- `-c, --count` print match counts per file
- `-j, --jobs <N>` number of worker threads for parallel search
- `--hidden` include hidden files and directories
- `--no-ignore` do not respect `.gitignore`/ignore files
- `--no-color` disable colored output
- `--layout` show a terminal layout of available functions
- `-h, --help` show help
- `-V, --version` show version

### Exit codes

- `0` match(es) found, or layout shown successfully
- `1` no matches found
- `2` error (invalid arguments/pattern, I/O errors, etc.)

## scout-convert

### Supported conversions

| Input | Output | Engine |
| --- | --- | --- |
| PNG, JPG, WebP, BMP, TIFF, GIF | PDF (one page per image) | native |
| TXT, CSV, JSON, Markdown | PDF | native |
| DOCX | PDF, HTML, Markdown | native (`rdocx`) |
| XLSX, PPTX | PDF | native (`office2pdf`) |
| PDF | TXT | native (`pdf-extract`) |
| PDF | PNG, JPG | native (`pdfium`, external DLL) |

Multiple images passed with `-o out.pdf` are merged into one PDF. Use
`--no-merge` to convert them individually instead.

PDF to DOCX and OCR are not supported.

### Examples

```bash
# Image to PDF
scout-convert photo.png

# Merge several images into one PDF
scout-convert page1.png page2.png page3.png -o book.pdf

# Text or Markdown to PDF
scout-convert notes.txt
scout-convert README.md -o README.pdf

# Office documents to PDF
scout-convert report.docx
scout-convert budget.xlsx slides.pptx --out-dir out

# PDF to text
scout-convert document.pdf -t txt

# PDF to images (150 DPI by default)
scout-convert document.pdf -t png --dpi 200
scout-convert document.pdf -t jpg --quality 85 -o page.jpg

# DOCX to HTML or Markdown
scout-convert report.docx -t html
scout-convert report.docx -t md

# Batch a directory, preview first
scout-convert docs/ --out-dir converted --dry-run
scout-convert docs/ --out-dir converted
```

### Options

- `INPUT...` files or directories (directories are walked for supported files)
- `-t, --to <FORMAT>` target format: `pdf`, `png`, `jpg`, `txt`, `html`, `md`
- `-o, --output <FILE>` output file for a single input (or merged PDF)
- `--out-dir <DIR>` output directory, one file per input
- `--merge` / `--no-merge` control merging multiple images into one PDF
- `--page-size <a4|letter|legal|a3>` PDF page size (default `a4`)
- `--dpi <N>` rasterization DPI for PDF to image (default `150`)
- `--quality <N>` JPEG quality from 1 to 100 (default `90`)
- `--engine <auto|native|libreoffice|pandoc|magick>` conversion engine
  (default `auto`)
- `-j, --jobs <N>` number of parallel conversions
- `--overwrite` overwrite existing outputs
- `--dry-run` print planned conversions without writing files
- `--list-formats` print the conversion matrix and exit
- `-q, --quiet` suppress per-file progress output
- `-v, --verbose` print extra information

If `--to` is omitted, a known output extension is used; otherwise images, text
and Office inputs default to PDF.

### Engines

`auto` tries the native implementation first and then falls back to an
installed external tool when one can handle the conversion:

- **LibreOffice** (`soffice`) — Office to PDF/HTML
- **Pandoc** — DOCX/Markdown/text to HTML/Markdown
- **ImageMagick** (`magick`) — image to image

Pass `--engine native`, `--engine libreoffice`, `--engine pandoc` or
`--engine magick` to force one path.

### PDF rasterization setup

PDF to PNG/JPG uses PDFium and needs `pdfium.dll`, which is not committed.
Fetch it once:

```powershell
./scripts/fetch-pdfium.ps1
```

The loader searches, in order:

1. `SCOUT_PDFIUM_PATH` (a DLL path or a directory containing it)
2. Next to `scout-convert.exe`
3. `vendor/pdfium/x64` next to the executable
4. `crates/scout-convert/vendor/pdfium/x64` (development builds)
5. The system library search path

When the library is unavailable, PDF rasterization and its tests are skipped.

### Exit codes

- `0` all conversions succeeded
- `1` some conversions failed, others succeeded
- `2` every conversion failed, or invalid arguments/inputs

### Limitations

- Text-to-PDF uses the built-in Courier font (WinAnsi encoding). Characters
  outside Latin-1 are written as `?`.
- Markdown is rendered as wrapped plain text for PDF output; styling is not
  preserved.
- `office2pdf` conversion is layout-aware but not identical to Microsoft Office
  rendering. Complex spreadsheets and presentations are best-effort.
- PDF to DOCX and OCR are out of scope.
- The PDFium DLL must match the target architecture (x64).

## Workspace layout

```text
crates/
  scout-core/     # search engine library
    src/{lib,output,search,walk}.rs
  scout/          # search CLI
    src/{main,cli,layout}.rs
  scout-convert/  # converter CLI
    src/{main,cli,format,plan}.rs
    src/engine/{mod,external}.rs
    src/engine/native/{mod,image_pdf,text_pdf,pdf_text,pdf_raster,office,docx_export}.rs
    tests/{cli,office}.rs
scripts/
  fetch-pdfium.ps1
```

`scout-core` owns matching, file walking and result formatting. `scout` maps
CLI arguments onto `SearchOptions`, `WalkOptions` and `PrintOptions`.
`scout-convert` plans jobs in `plan.rs`, detects formats in `format.rs`, and
dispatches to native or external engines in `engine/`.

## Testing

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Unit tests cover format detection, planning, and text layout. Integration tests
cover text/PDF round-trips, image merging, batch conversion, Office exports and
PDF rasterization (when PDFium is present).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Third-party dependencies and the PDFium DLL are described in
[THIRD_PARTY.md](THIRD_PARTY.md).
