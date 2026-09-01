use std::fs;

use chi_log::{Error, Event, SeriesParsed, SourceFetched};

/// sha256 of fixtures/ocpi/current/H100_SXM.json
const H100_SHA: &str = "608fb5ff229d86b8bb4ac6f4af6170c48126f0dbdb47c11c774428cac455b95f";

fn source_fetched() -> Event {
    Event::SourceFetched(SourceFetched {
        fetched_at: "2026-08-22T21:03:21.660Z".to_string(),
        source_url: "https://api.ornnai.com/api/gpu/H100%20SXM".to_string(),
        http_status: 200,
        series: "ocpi.current".to_string(),
        raw_sha256: H100_SHA.to_string(),
    })
}

fn series_parsed() -> Event {
    Event::SeriesParsed(SeriesParsed {
        series: "ocpi.current".to_string(),
        gpu_name: "H100 SXM".to_string(),
        index_value: "2.63".to_string(),
        valid_on: None,
        raw_sha256: H100_SHA.to_string(),
    })
}

#[test]
fn two_line_fixture_is_source_fetched_then_series_parsed_same_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("events.jsonl");
    let fetched = source_fetched();
    let parsed = series_parsed();
    let body = format!(
        "{}\n{}\n",
        serde_json::to_string(&fetched).unwrap(),
        serde_json::to_string(&parsed).unwrap()
    );
    fs::write(&path, body).unwrap();

    let events = chi_log::read_events(&path).expect("two-line jsonl");
    assert_eq!(events, vec![fetched, parsed]);
    match (&events[0], &events[1]) {
        (Event::SourceFetched(a), Event::SeriesParsed(b)) => {
            assert_eq!(a.raw_sha256, b.raw_sha256);
            assert_eq!(a.raw_sha256, H100_SHA);
        }
        other => panic!("expected SourceFetched then SeriesParsed, got {other:?}"),
    }
}

#[test]
fn truncated_line_is_invalid_line() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("events.jsonl");
    fs::write(&path, r#"{"type":"SourceFetched""#).unwrap();
    match chi_log::read_events(&path) {
        Err(Error::InvalidLine { line, .. }) => assert_eq!(line, 1),
        other => panic!("expected InvalidLine, got {other:?}"),
    }
}

#[test]
fn unknown_event_type_is_invalid_line() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("events.jsonl");
    fs::write(&path, "{\"type\":\"Nope\"}\n").unwrap();
    match chi_log::read_events(&path) {
        Err(Error::InvalidLine { line, .. }) => assert_eq!(line, 1),
        other => panic!("expected InvalidLine for unknown type, got {other:?}"),
    }
}

#[test]
fn empty_lines_are_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("events.jsonl");
    let fetched = source_fetched();
    let parsed = series_parsed();
    let body = format!(
        "{}\n\n{}\n",
        serde_json::to_string(&fetched).unwrap(),
        serde_json::to_string(&parsed).unwrap()
    );
    fs::write(&path, body).unwrap();
    let events = chi_log::read_events(&path).expect("blank line skipped");
    assert_eq!(events, vec![fetched, parsed]);
}
