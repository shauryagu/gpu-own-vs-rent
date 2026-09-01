use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, ValueEnum};

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Json,
}

#[derive(Args, Debug)]
pub struct ReplayArgs {
    /// Directory with events.jsonl and cas/{sha256}.
    #[arg(long)]
    log_dir: PathBuf,

    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
}

pub fn run(args: ReplayArgs) -> Result<()> {
    let ReplayArgs {
        log_dir,
        format: OutputFormat::Json,
    } = args;
    let events = chi_log::read_events(&log_dir.join("events.jsonl"))?;
    let catalog = chi_log::fold(&events, &log_dir)?;
    let bytes = chi_log::catalog_bytes(&catalog)?;
    std::io::stdout().write_all(&bytes)?;
    Ok(())
}
