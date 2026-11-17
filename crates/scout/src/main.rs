use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

mod cli;
mod layout;

use cli::Args;
use scout_core::{
    collect_files, search_files, search_stdin, Matcher, PrintOptions, Printer, SearchOptions,
    WalkOptions,
};

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(found) => {
            if found {
                ExitCode::from(0)
            } else {
                ExitCode::from(1)
            }
        }
        Err(err) => {
            eprintln!("scout: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Args) -> Result<bool> {
    if args.layout || args.pattern.is_none() {
        layout::print_layout(args.no_color)?;
        return Ok(true);
    }

    let matcher = Matcher::new(&SearchOptions {
        pattern: args.pattern.clone(),
        regex: args.regex,
        ignore_case: args.ignore_case,
    })?;
    let printer = Printer::new(&PrintOptions {
        files_with_matches: args.files_with_matches,
        count: args.count,
        no_color: args.no_color,
    });

    if args.path.as_os_str() == "-" {
        return search_stdin(&matcher, &printer);
    }

    let files = collect_files(&WalkOptions {
        path: args.path.clone(),
        globs: args.globs.clone(),
        hidden: args.hidden,
        no_ignore: args.no_ignore,
    })?;

    if let Some(jobs) = args.jobs {
        if jobs > 0 {
            let pool = rayon_thread_pool(jobs)?;
            return pool.install(|| search_files(files, &matcher, &printer));
        }
    }

    search_files(files, &matcher, &printer)
}

fn rayon_thread_pool(jobs: usize) -> Result<rayon::ThreadPool> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .map_err(Into::into)
}
