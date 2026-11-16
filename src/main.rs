use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

mod cli;
mod output;
mod search;
mod walk;

use cli::Args;

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
        output::print_layout(args.no_color)?;
        return Ok(true);
    }

    let matcher = search::Matcher::new(&args)?;
    let printer = output::Printer::new(&args);
    let files = walk::collect_files(&args)?;

    if let Some(jobs) = args.jobs {
        if jobs > 0 {
            let pool = rayon::ThreadPoolBuilder::new().num_threads(jobs).build()?;
            return pool.install(|| search::search_files(files, &matcher, &printer));
        }
    }

    search::search_files(files, &matcher, &printer)
}
