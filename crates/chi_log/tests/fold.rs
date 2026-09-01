use std::fs;
use std::path::{Path, PathBuf};

use chi_log::{catalog_bytes, fold, Error, Event, SeriesParsed, SourceFetched};

/// sha256 of fixtures/ocpi/current/H100_SXM.json
const H100_SHA: &str = "608fb5ff229d86b8bb4ac6f4af6170c48126f0dbdb47c11c774428cac455b95f";
/// sha256("hello")
const HELLO_SHA: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

fn log_v1() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/log/v1")
}

fn write_file(dir: &Path, rel: &str, body: &[u8]) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn source_fetched(raw_sha256: &str) -> Event {
    Event::SourceFetched(SourceFetched {
        fetched_at: "2026-08-22T21:03:21.660Z".to_string(),
        source_url: "https://api.ornnai.com/api/gpu/H100%20SXM".to_string(),
        http_status: 200,
        series: "ocpi.current".to_string(),
        raw_sha256: raw_sha256.to_string(),
    })
}

fn series_parsed(raw_sha256: &str) -> Event {
    Event::SeriesParsed(SeriesParsed {
        series: "ocpi.current".to_string(),
        gpu_name: "H100 SXM".to_string(),
        index_value: "2.63".to_string(),
        valid_on: None,
        raw_sha256: raw_sha256.to_string(),
    })
}

#[test]
fn fold_v1_fixture_matches_golden_catalog() {
    let log_dir = log_v1();
    let events = chi_log::read_events(&log_dir.join("events.jsonl")).expect("v1 events");
    let catalog = fold(&events, &log_dir).expect("fold v1");
    let bytes = catalog_bytes(&catalog).expect("catalog bytes");

    let golden = fs::read(log_dir.join("catalog.json")).expect("golden catalog.json");
    assert_eq!(bytes, golden);

    let json = String::from_utf8(bytes.clone()).expect("utf8 catalog");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("catalog json");
    let entry = &value["entries"][0];
    assert_eq!(entry["series"], "ocpi.current");
    assert_eq!(entry["gpu_name"], "H100 SXM");
    assert_eq!(entry["index_value"], "2.63");
    assert!(entry["valid_on"].is_null());
    assert_eq!(entry["raw_sha256"], H100_SHA);
    assert!(entry["parse_error"].is_null());
    assert!(!json.contains("leftover"));
    assert!(!json.contains("implied_salvage"));
    assert!(!json.contains("fair_rent"));
    assert!(!json.contains("daily-index"));
}

#[test]
fn fold_v1_twice_yields_equal_bytes() {
    let log_dir = log_v1();
    let events = chi_log::read_events(&log_dir.join("events.jsonl")).expect("v1 events");
    let first = catalog_bytes(&fold(&events, &log_dir).expect("first fold")).expect("first bytes");
    let second =
        catalog_bytes(&fold(&events, &log_dir).expect("second fold")).expect("second bytes");
    assert_eq!(first, second);
}

#[test]
fn source_fetched_without_parse_still_listed_with_null_parse_fields() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{HELLO_SHA}"), b"hello");
    let events = [source_fetched(HELLO_SHA)];
    let catalog = fold(&events, tmp.path()).expect("fetch without parse");
    let bytes = catalog_bytes(&catalog).expect("catalog bytes");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("catalog json");
    let entry = &value["entries"][0];
    let obj = entry.as_object().expect("entry object");
    assert!(obj.contains_key("gpu_name"), "gpu_name must be present");
    assert!(
        obj.contains_key("index_value"),
        "index_value must be present"
    );
    assert!(
        obj.contains_key("parse_error"),
        "parse_error must be present"
    );
    assert!(entry["gpu_name"].is_null());
    assert!(entry["index_value"].is_null());
    assert!(entry["parse_error"].is_null());
    assert_eq!(entry["series"], "ocpi.current");
    assert_eq!(entry["raw_sha256"], HELLO_SHA);
    assert_eq!(value["entries"].as_array().map(Vec::len), Some(1));
}

#[test]
fn missing_cas_blob_is_missing_cas() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("cas")).unwrap();
    let events = [source_fetched(H100_SHA)];
    match fold(&events, tmp.path()) {
        Err(Error::MissingCas { raw_sha256 }) => assert_eq!(raw_sha256, H100_SHA),
        other => panic!("expected MissingCas, got {other:?}"),
    }
}

#[test]
fn mismatched_cas_bytes_are_cas_hash_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{H100_SHA}"), b"wrong-bytes");
    let events = [source_fetched(H100_SHA)];
    match fold(&events, tmp.path()) {
        Err(Error::CasHashMismatch { raw_sha256, actual }) => {
            assert_eq!(raw_sha256, H100_SHA);
            assert_ne!(actual, H100_SHA);
        }
        other => panic!("expected CasHashMismatch, got {other:?}"),
    }
}

#[test]
fn invalid_cas_key_on_fetch_is_invalid_cas_key() {
    let tmp = tempfile::tempdir().unwrap();
    let events = [source_fetched("abc")];
    match fold(&events, tmp.path()) {
        Err(Error::InvalidCasKey(key)) => assert_eq!(key, "abc"),
        other => panic!("expected InvalidCasKey, got {other:?}"),
    }
}

#[test]
fn orphan_series_parsed_is_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let catalog = fold(&[series_parsed(HELLO_SHA)], tmp.path()).expect("orphan parse");
    assert!(catalog.entries.is_empty());
}

#[test]
fn second_series_parsed_on_same_hash_is_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{HELLO_SHA}"), b"hello");
    let extra = Event::SeriesParsed(SeriesParsed {
        series: "ocpi.current".to_string(),
        gpu_name: "other".to_string(),
        index_value: "9.99".to_string(),
        valid_on: None,
        raw_sha256: HELLO_SHA.to_string(),
    });
    let events = [source_fetched(HELLO_SHA), series_parsed(HELLO_SHA), extra];
    let catalog = fold(&events, tmp.path()).expect("second parse ignored");
    assert_eq!(catalog.entries.len(), 1);
    assert_eq!(catalog.entries[0].gpu_name.as_deref(), Some("H100 SXM"));
    assert_eq!(catalog.entries[0].index_value.as_deref(), Some("2.63"));
}

#[test]
fn parse_fills_latest_unmatched_fetch_with_same_hash() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{HELLO_SHA}"), b"hello");
    let first = Event::SourceFetched(SourceFetched {
        fetched_at: "2026-08-22T21:03:21.660Z".to_string(),
        source_url: "https://api.ornnai.com/api/gpu/H100%20SXM".to_string(),
        http_status: 200,
        series: "ocpi.current".to_string(),
        raw_sha256: HELLO_SHA.to_string(),
    });
    let second = Event::SourceFetched(SourceFetched {
        fetched_at: "2026-08-22T22:00:00.000Z".to_string(),
        source_url: "https://api.ornnai.com/api/gpu/H100%20SXM".to_string(),
        http_status: 200,
        series: "ocpi.current".to_string(),
        raw_sha256: HELLO_SHA.to_string(),
    });
    let events = [first, second, series_parsed(HELLO_SHA)];
    let catalog = fold(&events, tmp.path()).expect("latest unmatched");
    assert_eq!(catalog.entries.len(), 2);
    assert!(catalog.entries[0].gpu_name.is_none());
    assert!(catalog.entries[0].index_value.is_none());
    assert_eq!(catalog.entries[1].gpu_name.as_deref(), Some("H100 SXM"));
    assert_eq!(catalog.entries[1].fetched_at, "2026-08-22T22:00:00.000Z");
}
