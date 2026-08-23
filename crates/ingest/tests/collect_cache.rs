use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ingest::cache::{compact_fetched_at, sha256_hex, RawCache};
use ingest::error::IngestError;
use ingest::http::FixtureHttp;
use ingest::ocpi_current::collect_current;
use serde_json::Value;
use time::{Date, Month, OffsetDateTime, Time, UtcOffset};

fn frozen_at(nano: u32) -> OffsetDateTime {
    OffsetDateTime::new_in_offset(
        Date::from_calendar_date(2026, Month::August, 22).unwrap(),
        Time::from_hms_nano(4, 3, 1, nano).unwrap(),
        UtcOffset::UTC,
    )
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/ocpi")
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

fn jsonl_path(tmp: &Path) -> PathBuf {
    tmp.join("data/ocpi-hourly.jsonl")
}

fn jsonl_records(tmp: &Path) -> Vec<Value> {
    let text = fs::read_to_string(jsonl_path(tmp)).unwrap_or_default();
    text.lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn non_200_does_not_append_jsonl() {
    let tmp = tempfile::tempdir().unwrap();
    let fx = tmp.path().join("fx");
    write_file(
        &fx,
        "gpu-types-free.json",
        br#"{"success":true,"data":[{"gpu_name":"H100 SXM","region":""}]}"#,
    );

    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(&fx);
    let err = collect_current(frozen_at(0), &http, &cache).unwrap_err();
    assert!(matches!(err, IngestError::CollectFailed { count: 1, .. }));
    assert!(jsonl_records(tmp.path()).is_empty());
}

#[test]
fn status_200_writes_raw_bytes_bit_identical() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(fixtures_dir());
    collect_current(frozen_at(0), &http, &cache).unwrap();

    let fixture = fs::read(fixtures_dir().join("current/H100_SXM.json")).unwrap();
    let raw_path = tmp.path().join(format!(
        "data/raw/ocpi/current/H100_SXM/{}.json",
        compact_fetched_at(frozen_at(0))
    ));
    let written = fs::read(&raw_path).unwrap();
    assert_eq!(written, fixture);

    let record = jsonl_records(tmp.path())
        .into_iter()
        .find(|rec| rec["gpu_name"] == "H100 SXM")
        .unwrap();
    assert_eq!(record["raw_sha256"], sha256_hex(&fixture));
    assert_eq!(record["schema"], "ocpi.hourly.v1");
    assert_eq!(record["series"], "ocpi.current");
    assert_eq!(record["valid_on"], Value::Null);
}

#[test]
fn second_collect_appends_does_not_rewrite_first_line() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(fixtures_dir());

    collect_current(frozen_at(0), &http, &cache).unwrap();
    let after_first = fs::read_to_string(jsonl_path(tmp.path())).unwrap();
    collect_current(frozen_at(1), &http, &cache).unwrap();
    let after_second = fs::read_to_string(jsonl_path(tmp.path())).unwrap();

    assert!(after_second.starts_with(&after_first));
    assert!(after_second.len() > after_first.len());
    assert_eq!(jsonl_records(tmp.path()).len(), 10);
}

#[test]
fn frozen_fetched_at_appears_verbatim() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(fixtures_dir());
    collect_current(frozen_at(0), &http, &cache).unwrap();

    for record in jsonl_records(tmp.path()) {
        assert_eq!(record["fetched_at"], "2026-08-22T04:03:01.000Z");
    }
}

#[test]
fn empty_gpu_types_free_is_error() {
    let tmp = tempfile::tempdir().unwrap();
    let fx = tmp.path().join("fx");
    write_file(
        &fx,
        "gpu-types-free.json",
        br#"{"success":true,"access":"free-tier","data":[]}"#,
    );

    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(&fx);
    let err = collect_current(frozen_at(0), &http, &cache).unwrap_err();
    assert!(matches!(err, IngestError::EmptyGpuList));
}

#[test]
fn current_gpu_name_mismatch_does_not_append_jsonl() {
    let tmp = tempfile::tempdir().unwrap();
    let fx = tmp.path().join("fx");
    write_file(
        &fx,
        "gpu-types-free.json",
        br#"{"success":true,"data":[{"gpu_name":"H100 SXM","region":""}]}"#,
    );
    write_file(
        &fx,
        "current/H100_SXM.json",
        br#"{"success":true,"data":{"gpu_name":"B200","region":"","index_value":2.63,"last_updated":"2026-08-22T21:03:21.660Z"}}"#,
    );

    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(&fx);
    let err = collect_current(frozen_at(0), &http, &cache).unwrap_err();
    assert!(matches!(err, IngestError::CollectFailed { count: 1, .. }));
    assert!(
        jsonl_records(tmp.path()).is_empty(),
        "mismatched gpu_name must not become an H100 current print"
    );
    let raw_dir = tmp.path().join("data/raw/ocpi/current/H100_SXM");
    assert!(
        raw_dir.read_dir().unwrap().next().is_some(),
        "raw body is still cached"
    );
}

#[test]
fn overlapping_collect_fails_without_writing() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    fs::create_dir_all(&data).unwrap();
    let lock_path = data.join("collect.lock");
    let lock = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    lock.lock().expect("hold collect lock");

    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(fixtures_dir());
    let err = collect_current(frozen_at(0), &http, &cache).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("already running") || msg.contains("lock"),
        "expected lock failure, got {msg}"
    );
    assert!(jsonl_records(tmp.path()).is_empty());
}

#[test]
fn successful_raw_write_leaves_no_tmp_sibling() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(fixtures_dir());
    collect_current(frozen_at(0), &http, &cache).unwrap();

    let raw_root = tmp.path().join("data/raw");
    let mut tmps = Vec::new();
    visit_tmps(&raw_root, &mut tmps);
    assert!(tmps.is_empty(), "leftover tmp files: {tmps:?}");
}

fn visit_tmps(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            visit_tmps(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("tmp")
            || path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".tmp"))
        {
            out.push(path);
        }
    }
}

#[test]
fn same_utc_second_writes_distinct_raw_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = cache_for(tmp.path());
    let http = FixtureHttp::new(fixtures_dir());

    let t1 = frozen_at(0);
    let t2 = frozen_at(123_456_789);
    collect_current(t1, &http, &cache).unwrap();
    collect_current(t2, &http, &cache).unwrap();

    let p1 = tmp.path().join(format!(
        "data/raw/ocpi/current/H100_SXM/{}.json",
        compact_fetched_at(t1)
    ));
    let p2 = tmp.path().join(format!(
        "data/raw/ocpi/current/H100_SXM/{}.json",
        compact_fetched_at(t2)
    ));
    assert_eq!(p1.file_name().unwrap(), "2026-08-22T040301000000000Z.json");
    assert_eq!(p2.file_name().unwrap(), "2026-08-22T040301123456789Z.json");
    assert!(p1.exists());
    assert!(p2.exists());
    assert_ne!(p1, p2);
}
