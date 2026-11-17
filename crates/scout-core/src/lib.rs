//! Core search engine for the `scout` CLI.

pub mod output;
pub mod search;
pub mod walk;

pub use output::{set_color, Match, PrintOptions, Printer, SearchResult};
pub use search::{search_files, search_stdin, Matcher, SearchOptions};
pub use walk::{collect_files, WalkOptions};
