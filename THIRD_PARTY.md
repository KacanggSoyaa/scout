# Third-party software

`scout` is licensed under MIT OR Apache-2.0. It depends on the following
crates. Transitive dependencies and exact versions are recorded in
`Cargo.lock`; each crate ships its own license text in the cargo registry.

## Direct dependencies

| Crate | Version | License |
| --- | --- | --- |
| anyhow | 1 | MIT OR Apache-2.0 |
| clap | 4 | MIT OR Apache-2.0 |
| image | 0.25 | MIT OR Apache-2.0 |
| pdf-extract | 0.12 | MIT |
| printpdf | 0.12 | MIT |
| pulldown-cmark | 0.13 | MIT |
| rayon | 1 | MIT OR Apache-2.0 |
| termcolor | 1 | MIT OR Unlicense |
| walkdir | 2 | MIT OR Unlicense |
| ignore | 0.4 | MIT OR Unlicense (via `scout-core`) |
| memchr | 2 | MIT OR Unlicense (via `scout-core`) |
| regex | 1 | MIT OR Apache-2.0 (via `scout-core`) |
| office2pdf | 0.11 | Apache-2.0 |
| rdocx | 0.17 | MIT OR Apache-2.0 |
| pdfium-render | 0.9 | MIT OR Apache-2.0 |

## Native libraries

- **PDFium** (`pdfium.dll`) is used by `scout-convert` for PDF rasterization.
  It is not bundled in this repository. Run `scripts/fetch-pdfium.ps1` or point
  `SCOUT_PDFIUM_PATH` at an existing copy. PDFium is distributed under the
  BSD-3-Clause license by the PDFium project.

## Fonts

`office2pdf` may embed fonts in generated PDFs. Refer to the `office2pdf`
crate for the licenses of its bundled fonts. Text-to-PDF conversion in
`scout-convert` uses the PDF standard fonts Helvetica/Courier and does not
embed external font files.
