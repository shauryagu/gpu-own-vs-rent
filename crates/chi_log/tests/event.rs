use chi_log::{Event, SeriesParsed, SourceFetched};

#[test]
fn source_fetched_json_round_trips() {
    let event = Event::SourceFetched(SourceFetched {
        fetched_at: "2026-08-22T21:03:21.660Z".to_string(),
        source_url: "https://api.ornnai.com/api/gpu/H100%20SXM".to_string(),
        http_status: 200,
        series: "ocpi.current".to_string(),
        raw_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    });
    let json = serde_json::to_string(&event).expect("serialize SourceFetched");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json object");
    assert_eq!(value["type"], "SourceFetched");
    assert_eq!(value["series"], "ocpi.current");
    assert_eq!(value["http_status"], 200);
    let back: Event = serde_json::from_str(&json).expect("deserialize SourceFetched");
    assert_eq!(event, back);
}

#[test]
fn series_parsed_json_round_trips() {
    let event = Event::SeriesParsed(SeriesParsed {
        series: "ocpi.current".to_string(),
        gpu_name: "H100 SXM".to_string(),
        index_value: "2.63".to_string(),
        valid_on: None,
        raw_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    });
    let json = serde_json::to_string(&event).expect("serialize SeriesParsed");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json object");
    assert_eq!(value["type"], "SeriesParsed");
    assert_eq!(value["series"], "ocpi.current");
    assert_eq!(value["index_value"], "2.63");
    assert!(value["valid_on"].is_null());
    let back: Event = serde_json::from_str(&json).expect("deserialize SeriesParsed");
    assert_eq!(event, back);
}

#[test]
fn unknown_event_type_nope_fails_closed() {
    let json = r#"{"type":"Nope"}"#;
    let err = serde_json::from_str::<Event>(json).expect_err("unknown type must error");
    assert!(
        err.to_string().contains("Nope"),
        "error should name the unknown type, got {err}"
    );
}
