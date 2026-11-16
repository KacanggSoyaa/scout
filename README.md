# scout

A fast, concurrent CLI file searcher in Rust (literal-first, grep/find-like).

## Build

```bash
cargo build --release
```

> On Windows with the MSVC Rust toolchain, install **Visual Studio Build Tools** with “Desktop development with C++” (provides `link.exe`). Without a linker, `cargo build`/`cargo run` will fail.

## Usage

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

# Run without installing
cargo run -- "pattern" .
cargo run -- --layout
```

## Options

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

## Exit codes

- `0` match(es) found, or layout shown successfully
- `1` no matches found
- `2` error (invalid arguments/pattern, I/O errors, etc.)

## CLI and function overview

### `src/cli.rs`
Defines command-line interface configuration using `clap`.

- **`Args` (struct)**: Parsed CLI arguments and flags.
  - `pattern: Option<String>` — text/regex pattern to search for; `None` triggers layout display.
  - `path: PathBuf` — target file or directory (default `.`).
  - `regex: bool` — interpret `pattern` as a regex instead of a literal.
  - `ignore_case: bool` — enable case-insensitive matching.
  - `globs: Vec<String>` — optional include globs (can be passed multiple times).
  - `files_with_matches: bool` — output only file paths that contain matches (`-l`).
  - `count: bool` — output match counts per file (`-c`).
  - `jobs: Option<usize>` — thread count for Rayon parallel search.
  - `hidden: bool` — include hidden files/directories.
  - `no_ignore: bool` — disable ignore-file filtering.
  - `no_color: bool` — disable colored terminal output.
  - `layout: bool` — explicitly show the terminal functions/usage layout.

### `src/main.rs`
Program entry point and high-level orchestration.

- **`main() -> ExitCode`**: Parses CLI args, runs the app, and maps outcomes to exit codes (`0` found/layout, `1` no matches, `2` error with message to stderr).
- **`run(args: Args) -> Result<bool>`**: Core workflow.
  1. If `--layout` or no pattern is provided, prints the layout and returns success.
  2. Builds a `Matcher` from args.
  3. Creates a `Printer` for output formatting.
  4. Collects searchable files via `walk::collect_files`.
  5. Searches in a custom Rayon thread pool when `--jobs > 0`, otherwise uses default parallel search.
  6. Returns whether any matches were found.

### `src/walk.rs`
File discovery and ignore/glob handling.

- **`collect_files(args: &Args) -> Result<Vec<PathBuf>>`**: Produces the list of files to search.
  - Configures `ignore::WalkBuilder` for hidden/ignore behavior.
  - Applies glob overrides when `-g/--glob` is provided.
  - If `PATH` is a file, returns just that file.
  - Walks directories and collects file entries, warning on walk errors.

### `src/search.rs`
Pattern matching and per-file searching.

- **`Matcher` (struct)**: Holds either a compiled `Regex` or a literal string matcher.
- **`Matcher::new(args: &Args) -> Result<Self>`**: Builds the matcher.
  - Uses regex mode when `--regex` or `--ignore-case` is set (escaping literal pattern for case-insensitive literal search).
  - Otherwise stores the raw literal pattern.
- **`Matcher::find_match(&self, line: &str) -> Option<(usize, String)>`**: Finds the first match in a line, returning 1-based column and matched text.
- **`search_files(files: Vec<PathBuf>, matcher: &Matcher, printer: &Printer) -> Result<bool>`**: Searches files in parallel with Rayon, collects successful results, sorts by path, prints non-empty results, and returns whether any matches were found.
- **`search_file(path: &Path, matcher: &Matcher) -> Result<Option<SearchResult>>`**: Searches a single file.
  - Probes for NUL bytes to skip likely binary files.
  - Reads lines and records matches with line/column/text.
  - Skips unreadable/non-text cases and returns `None` when no matches exist.

### `src/output.rs`
Terminal output formatting and layout display.

- **`Match` (struct)**: One match record (`line_no`, `col`, `text`, `matched`).
- **`SearchResult` (struct)**: All matches for a file path.
- **`Printer` (struct)**: Output mode settings derived from args (`files_with_matches`, `count`, `no_color`).
- **`Printer::new(args: &Args) -> Self`**: Creates a printer from CLI args.
- **`Printer::choice(&self) -> ColorChoice`**: Chooses colored vs plain output.
- **`Printer::print_result(&self, res: &SearchResult) -> Result<()>`**: Prints results in one of three modes:
  - file paths only (`-l`)
  - `path:count` (`-c`)
  - detailed `path:line:col:line_text` with highlighted match
- **`Printer::set(&self, out: &mut StandardStream, color: Color, bold: bool) -> Result<()>`**: Applies terminal color/style.
- **`print_layout(no_color: bool) -> Result<()>`**: Prints the terminal layout showing available functions, usage examples, and options.
- **`section(out: &mut StandardStream, name: &str) -> Result<()>`**: Renders a layout section header.
- **`item(out: &mut StandardStream, key: &str, desc: &str) -> Result<()>`**: Renders a labeled function/feature row in the layout.
