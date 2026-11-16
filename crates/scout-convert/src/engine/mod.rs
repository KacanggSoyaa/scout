pub mod external;
pub mod native;

use crate::cli::{Engine, PageSize};
use crate::plan::Job;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub engine: Engine,
    pub page_size: PageSize,
    pub dpi: u32,
    pub quality: u8,
    pub overwrite: bool,
    pub verbose: bool,
}

pub fn run(job: &Job, opts: &ConvertOptions) -> Result<()> {
    if let Some(parent) = job.output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    match opts.engine {
        Engine::Native => native::convert(job, opts),
        Engine::Libreoffice => external::libreoffice(job, opts),
        Engine::Pandoc => external::pandoc(job, opts),
        Engine::Magick => external::magick(job, opts),
        Engine::Auto => match native::convert(job, opts) {
            Ok(()) => Ok(()),
            Err(native_err) => {
                if opts.verbose {
                    eprintln!(
                        "scout-convert: native conversion failed: {native_err:#}; trying external tools"
                    );
                }
                match external::fallback(job, opts) {
                    Ok(()) => Ok(()),
                    Err(fallback_err) => {
                        if opts.verbose {
                            eprintln!(
                                "scout-convert: external fallback unavailable: {fallback_err:#}"
                            );
                        }
                        Err(native_err)
                    }
                }
            }
        },
    }
}
