//! Command-line argument definitions for `scout`.

use clap::{ArgAction, Parser};
use std::path::PathBuf;

/// Parsed command-line options controlling search behavior and output.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "scout",
    version,
    about = "A fast concurrent CLI file searcher",
    long_about = "Search files quickly with literal or regex matching, glob filters, and concurrent workers."
)]
pub struct Args {
    /// Pattern to search for (literal by default). Omit to show the terminal layout.
    pub pattern: Option<String>,

    /// File or directory to search
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Treat pattern as a regular expression
    #[arg(short = 'e', long = "regex")]
    pub regex: bool,

    /// Case-insensitive matching
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Include files matching glob (repeatable)
    #[arg(short = 'g', long = "glob", value_name = "GLOB", action = ArgAction::Append)]
    pub globs: Vec<String>,

    /// Print only names of files with matches
    #[arg(short = 'l', long = "files-with-matches")]
    pub files_with_matches: bool,

    /// Print match counts per file
    #[arg(short = 'c', long = "count")]
    pub count: bool,

    /// Number of worker threads
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<usize>,

    /// Search hidden files and directories
    #[arg(long = "hidden")]
    pub hidden: bool,

    /// Do not respect .gitignore and ignore files
    #[arg(long = "no-ignore")]
    pub no_ignore: bool,

    /// Disable colored output
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Show a terminal layout of available functions
    #[arg(long = "layout")]
    pub layout: bool,
}
