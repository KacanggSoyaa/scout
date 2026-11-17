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
echo "TODO: fix me" | scout TODO -   # read from stdin
```

### Options

- `pattern` optional search pattern (literal by default); omit to show the layout
- `PATH` file or directory to search (default: `.`); use `-` to read stdin
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

`scout-convert` converts images, text/Markdown, Office documents and PDFs. Most
conversions run entirely in-process; LibreOffice, Pandoc and ImageMagick are
used only as fallbacks when installed.

### Install for everyday use

```powershell
cargo install --path crates/scout-convert
```

This puts `scout-convert.exe` in `~/.cargo/bin` (already on PATH), so you can run
the examples below from any directory. Re-run with `--force` after changing the
source. Without installing, run them from the repo root as
`cargo run -p scout-convert -- ...` instead.

### Quick start

```powershell
# 1. Build the workspace
cargo build --release

# 2. Optional: enable PDF -> PNG/JPG (downloads pdfium.dll once)
./scripts/fetch-pdfium.ps1

# 3. Convert. Output goes next to the input by default.
.\target\release\scout-convert.exe photo.png        # -> photo.pdf
.\target\release\scout-convert.exe notes.txt        # -> notes.pdf
.\target\release\scout-convert.exe report.docx -t md -o report.md
```

### How outputs are chosen

| Invocation | Result |
| --- | --- |
| `scout-convert a.png` | `a.pdf` in the same directory |
| `scout-convert a.txt -o out/manual.pdf` | `out/manual.pdf` (directories are created) |
| `scout-convert docs/ --out-dir out` | `out/<name>.pdf` for every supported file under `docs/` |
| `scout-convert a.png b.png -o book.pdf` | one `book.pdf` with two pages |
| `scout-convert a.png b.png --no-merge` | `a.pdf` and `b.pdf` |

- Directories are searched recursively; unsupported files are skipped.
- Existing outputs are never replaced unless you pass `--overwrite`.
- When `--to` is omitted, the output extension decides the target. Images, text
  and Office inputs default to PDF; PDF inputs need `--to` or an output name
  such as `-o page.png`.
- Merging happens automatically when several images are passed together with
  `-o <file>.pdf`. `--merge` forces it (output defaults to `merged.pdf`);
  `--no-merge` converts each image separately.
- `--dry-run` prints the plan without writing anything.
- `--out-dir` flattens directory trees, so two inputs with the same file stem
  map to the same output path. Rename or convert them separately.

### Supported conversions

| Input | Output | Engine |
| --- | --- | --- |
| PNG, JPG, WebP, BMP, TIFF, GIF | PDF (one page per image) | native |
| TXT, CSV, JSON, Markdown | PDF | native |
| DOCX | PDF, HTML, Markdown | native (`rdocx`) |
| XLSX, PPTX | PDF | native (`office2pdf`) |
| PDF | TXT | native (`pdf-extract`) |
| PDF | PNG, JPG | native (`pdfium`, external DLL) |

PDF to DOCX and OCR are not supported.

### Common recipes

```bash
# Images -> PDF
scout-convert photo.png
scout-convert scan-1.jpg scan-2.jpg scan-3.jpg -o scans.pdf

# Text, CSV, JSON, Markdown -> PDF
scout-convert notes.txt
scout-convert data.csv --page-size letter
scout-convert README.md -o README.pdf

# Office -> PDF
scout-convert report.docx
scout-convert budget.xlsx slides.pptx --out-dir out

# PDF -> text
scout-convert document.pdf -t txt

# PDF -> images (one file per page when a PDF has several pages)
scout-convert document.pdf -t png --dpi 200
scout-convert document.pdf -t jpg --quality 85 -o page.jpg

# DOCX -> HTML or Markdown
scout-convert report.docx -t html
scout-convert report.docx -t md

# Batch a directory, preview first
scout-convert docs/ --out-dir converted --dry-run
scout-convert docs/ --out-dir converted --jobs 4
```

Multi-page PDFs rasterize to `page-1.png`, `page-2.png`, and so on when the
output stem is `page.png`; with a single page the exact output name is used.

### Options

- `INPUT...` files or directories (directories are walked for supported files)
- `-t, --to <FORMAT>` target format: `pdf`, `png`, `jpg`, `txt`, `html`, `md`
- `-o, --output <FILE>` output file for a single input (or merged PDF)
- `--out-dir <DIR>` output directory, one file per input
- `--merge` / `--no-merge` control merging multiple images into one PDF
- `--page-size <a4|letter|legal|a3>` page size for image/text PDF output
  (default `a4`; Office conversions keep their original layout)
- `--dpi <N>` rasterization DPI for PDF to image (default `150`)
- `--quality <N>` JPEG quality from 1 to 100 (default `90`)
- `--engine <auto|native|libreoffice|pandoc|magick>` conversion engine
  (default `auto`)
- `-j, --jobs <N>` number of parallel conversions (`0` uses the Rayon default)
- `--overwrite` overwrite existing outputs
- `--dry-run` print planned conversions without writing files
- `--list-formats` print the conversion matrix and exit
- `-q, --quiet` suppress per-file success output
- `-v, --verbose` print fallback diagnostics when `auto` tries external tools
- `-h, --help` / `-V, --version` show help or version

### Engines

`auto` tries the native implementation first and then falls back to an
installed external tool when one can handle the conversion:

- **LibreOffice** (`soffice`) — Office to PDF/HTML
- **Pandoc** — DOCX/Markdown/text to HTML/Markdown
- **ImageMagick** (`magick`) — image to image

Pass `--engine native`, `--engine libreoffice`, `--engine pandoc` or
`--engine magick` to force one path. If the requested tool is missing or the
conversion pair is unsupported, the file fails with a message on stderr.

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

Errors for individual files are printed to stderr while the remaining files
continue converting.

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
