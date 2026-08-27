use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ingest::cache::{compact_fetched_at, RawCache};
use ingest::epoch::collect_epoch;
use ingest::http::FixtureHttp;
use ingest::ocpi_daily::collect_daily;
use time::{Date, Month, OffsetDateTime, Time, UtcOffset};

fn frozen_at() -> OffsetDateTime {
    OffsetDateTime::new_in_offset(
        Date::from_calendar_date(2026, Month::August, 22).unwrap(),
        Time::from_hms_nano(12, 0, 0, 0).unwrap(),
        UtcOffset::UTC,
    )
}

fn write_file(dir: &Path, rel: &str, body: &[u8]) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn cache_for(tmp: &Path) -> RawCache {
    RawCache::new(tmp.join("data")).with_pause(Duration::ZERO)
}

const RAW_DAILY: &[u8] = br#"{"success":true,"data":{"gpu_type":"H100 SXM","region":"global","index_value":2.879583333333333,"date":"2026-08-21T20:00:00.000Z"}}"#;
const RAW_ALL: &[u8] = br#"{"success":true,"date":"2026-08-21T20:00:00.000Z","data":[{"gpu_type":"H100 SXM","region":"global","index_value":2.879583333333333,"date":"2026-08-21T20:00:00.000Z"}],"count":1}"#;
const RAW_HISTORY: &[u8] = br#"{"success":true,"gpu_type":"H100 SXM","access":"public-3mo","data":[{"timestamp":"2026-08-21T20:00:00.000Z","index_value":2.88}]}"#;
const TYPES: &[u8] = br#"{"success":true,"data":[{"gpu_name":"H100 SXM","region":""}]}"#;

#[test]
fn collect_daily_writes_raw_envelopes_and_not_hourly_jsonl() {
    let tmp = tempfile::tempdir().unwrap();
    let fx = tmp.path().join("fx");
    write_file(&fx, "gpu-types-free.json", TYPES);
    write_file(&fx, "daily-index-all.json", RAW_ALL);
    write_file(&fx, "daily-index-http/H100_SXM.json", RAW_DAILY);
    write_file(&fx, "daily-history/H100_SXM.json", RAW_HISTORY);

    let cache = cache_for(tmp.path());
    collect_daily(frozen_at(), &FixtureHttp::new(&fx), &cache).unwrap();

    let stamp = compact_fetched_at(frozen_at());
    let daily = tmp
        .path()
        .join(format!("data/raw/ocpi/daily-index/H100_SXM/{stamp}.json"));
    let all = tmp
        .path()
        .join(format!("data/raw/ocpi/daily-index-all/{stamp}.json"));
    let history = tmp
        .path()
        .join(format!("data/raw/ocpi/daily-history/H100_SXM/{stamp}.json"));
    assert_eq!(fs::read(&daily).unwrap(), RAW_DAILY);
    assert_eq!(fs::read(&all).unwrap(), RAW_ALL);
    assert_eq!(fs::read(&history).unwrap(), RAW_HISTORY);
    assert!(!tmp.path().join("data/ocpi-hourly.jsonl").exists());
}

#[test]
fn collect_daily_gpu_type_mismatch_keeps_raw() {
    let tmp = tempfile::tempdir().unwrap();
    let fx = tmp.path().join("fx");
    write_file(&fx, "gpu-types-free.json", TYPES);
    write_file(&fx, "daily-index-all.json", RAW_ALL);
    write_file(
        &fx,
        "daily-index-http/H100_SXM.json",
        br#"{"success":true,"data":{"gpu_type":"H200","region":"global","index_value":2.879583333333333,"date":"2026-08-21T20:00:00.000Z"}}"#,
    );
    write_file(&fx, "daily-history/H100_SXM.json", RAW_HISTORY);

    let cache = cache_for(tmp.path());
    let err = collect_daily(frozen_at(), &FixtureHttp::new(&fx), &cache).unwrap_err();
    assert!(err.to_string().contains("gpu_type"));
    let stamp = compact_fetched_at(frozen_at());
    assert!(tmp
        .path()
        .join(format!("data/raw/ocpi/daily-index/H100_SXM/{stamp}.json"))
        .exists());
}

#[test]
fn collect_epoch_writes_raw_csv() {
    let tmp = tempfile::tempdir().unwrap();
    let fx = tmp.path().join("fx");
    let excerpt = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/epoch/ml_hardware.excerpt.csv");
    write_file(&fx, "ml_hardware.excerpt.csv", &fs::read(excerpt).unwrap());

    let cache = cache_for(tmp.path());
    collect_epoch(frozen_at(), &FixtureHttp::new(&fx), &cache).unwrap();

    let stamp = compact_fetched_at(frozen_at());
    let path = tmp
        .path()
        .join(format!("data/raw/epoch/ml_hardware/{stamp}.csv"));
    assert!(path.exists());
    assert!(!tmp.path().join("data/ocpi-hourly.jsonl").exists());
}
