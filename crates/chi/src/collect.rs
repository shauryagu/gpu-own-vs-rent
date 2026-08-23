use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Args;
use ingest::cache::RawCache;
use ingest::http::LiveHttp;
use ingest::ocpi_current::collect_current;
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
    match args.series {
        Series::Current => {
            let now = OffsetDateTime::now_utc();
            let http = LiveHttp::new()?;
            let cache = RawCache::new(args.data_dir);
            collect_current(now, &http, &cache)?;
            Ok(())
        }
        Series::Daily | Series::Epoch => bail!("not implemented"),
    }
}
