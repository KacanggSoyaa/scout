use anyhow::{bail, Result};
use clap::Parser;
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};

mod cli;
mod engine;
mod format;
mod plan;

use cli::Args;

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("scout-convert: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Args) -> Result<ExitCode> {
    if args.list_formats {
        print_formats();
        return Ok(ExitCode::SUCCESS);
    }

    let plan = plan::build(&args)?;

    if args.dry_run {
        for job in &plan.jobs {
            let inputs = job
                .inputs
                .iter()
                .map(|input| input.path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "{inputs} -> {} ({})",
                job.output.display(),
                job.target.label()
            );
        }
        return Ok(ExitCode::SUCCESS);
    }

    for job in &plan.jobs {
        if job.output.exists() && !args.overwrite {
            bail!(
                "output already exists: {} (use --overwrite)",
                job.output.display()
            );
        }
    }

    let opts = engine::ConvertOptions {
        engine: args.engine,
        page_size: args.page_size,
        dpi: args.dpi,
        quality: args.quality,
        overwrite: args.overwrite,
        verbose: args.verbose,
    };

    let pool = build_pool(args.jobs)?;
    let failures = AtomicUsize::new(0);

    pool.install(|| {
        use rayon::prelude::*;
        plan.jobs.par_iter().for_each(|job| {
            let inputs = job
                .inputs
                .iter()
                .map(|input| input.path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            match engine::run(job, &opts) {
                Ok(()) => {
                    if !args.quiet {
                        println!("{inputs} -> {}", job.output.display());
                    }
                }
                Err(err) => {
                    failures.fetch_add(1, Ordering::Relaxed);
                    eprintln!(
                        "scout-convert: {inputs} -> {}: {err:#}",
                        job.output.display()
                    );
                }
            }
        });
    });

    let failed = failures.load(Ordering::Relaxed);
    let total = plan.jobs.len();
    if failed == 0 {
        Ok(ExitCode::SUCCESS)
    } else if failed == total {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::from(1))
    }
}

fn build_pool(jobs: Option<usize>) -> Result<rayon::ThreadPool> {
    let mut builder = rayon::ThreadPoolBuilder::new();
    if let Some(count) = jobs {
        if count > 0 {
            builder = builder.num_threads(count);
        }
    }
    Ok(builder.build()?)
}

fn print_formats() {
    println!("Supported conversions:");
    println!("  images (png, jpg, webp, bmp, tiff, gif) -> pdf");
    println!("  txt, csv, json, md -> pdf");
    println!("  docx -> pdf, html, md");
    println!("  xlsx, pptx -> pdf");
    println!("  pdf -> txt, png, jpg");
    println!();
    println!("External fallbacks (--engine libreoffice|pandoc|magick):");
    println!("  LibreOffice: Office -> pdf/html");
    println!("  Pandoc: docx/md/txt -> html/md");
    println!("  ImageMagick: image conversions");
}
