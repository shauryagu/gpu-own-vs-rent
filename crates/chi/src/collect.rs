use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use ingest::cache::RawCache;
use ingest::epoch::collect_epoch;
use ingest::http::LiveHttp;
use ingest::ocpi_current::collect_current;
use ingest::ocpi_daily::collect_daily;
use time::OffsetDateTime;

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum Series {
    #[default]
    Current,
    Daily,
    Epoch,
}

#[derive(Args, Debug)]
pub struct CollectArgs {
    /// Directory for raw cache and JSONL.
    #[arg(long, default_value = "data")]
    data_dir: PathBuf,

    /// Which series to collect.
    #[arg(long, value_enum, default_value_t = Series::Current)]
    series: Series,
}

pub fn run(args: CollectArgs) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let http = LiveHttp::new()?;
    let cache = RawCache::new(args.data_dir);
    match args.series {
        Series::Current => collect_current(now, &http, &cache)?,
        Series::Daily => collect_daily(now, &http, &cache)?,
        Series::Epoch => collect_epoch(now, &http, &cache)?,
    }
    Ok(())
}
