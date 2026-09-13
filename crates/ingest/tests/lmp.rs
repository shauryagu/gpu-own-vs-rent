use std::path::PathBuf;

use rust_decimal::Decimal;
use time::{Date, Month, OffsetDateTime, Time, UtcOffset};

fn fixture_bytes(rel: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(rel);
    std::fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str_exact(s).expect("exact decimal")
}

#[test]
fn wrapper_keeps_mwh_source_token_and_converts_per_kwh() {
    let record =
        ingest::lmp::parse_lmp_wrapper(&fixture_bytes("energy/lmp/pjm_rto.json")).expect("wrapper");

    let fetched = OffsetDateTime::new_in_offset(
        Date::from_calendar_date(2026, Month::September, 10).unwrap(),
        Time::from_hms_nano(12, 0, 0, 0).unwrap(),
        UtcOffset::UTC,
    );
    assert_eq!(record.fetched_at.get(), fetched);
    assert_eq!(
        record.source_url,
        "https://www.pjm.com/markets-and-operations.aspx"
    );
    assert_eq!(record.iso, "PJM");
    assert_eq!(record.node, "RTO");
    assert_eq!(
        record.valid_on.get(),
        Date::from_calendar_date(2026, Month::September, 10).unwrap()
    );
    assert_eq!(record.lmp_usd_per_mwh, "49.24");
    assert_eq!(record.usd_per_kwh.amount(), dec("0.04924"));
}

#[test]
fn json_number_token_is_exact_and_not_the_teaching_print() {
    let record = ingest::lmp::parse_lmp_wrapper(
        br#"{
          "fetched_at": "2026-09-10T12:00:00.000Z",
          "source_url": "https://www.pjm.com/markets-and-operations.aspx",
          "body": {
            "iso": "PJM",
            "node": "WESTERN HUB",
            "unit": "USD/MWh",
            "lmp_usd_per_mwh": 33.17,
            "valid_on": "2026-09-10"
          }
        }"#,
    )
    .expect("json number");
    assert_eq!(record.lmp_usd_per_mwh, "33.17");
    assert_ne!(record.lmp_usd_per_mwh, "49.24");
    assert_eq!(record.usd_per_kwh.amount(), dec("0.03317"));
}

#[test]
fn mwh_divided_by_thousand_is_usd_per_kwh() {
    let pi = ingest::lmp::usd_per_kwh_from_mwh("33.17").expect("convert");
    assert_eq!(pi.amount(), dec("0.03317"));
    assert_ne!(pi.amount(), dec("0.04924"));
}

#[test]
fn rejects_wrong_unit() {
    let err = ingest::lmp::parse_lmp_wrapper(
        br#"{
          "fetched_at": "2026-09-10T12:00:00.000Z",
          "source_url": "https://www.pjm.com/markets-and-operations.aspx",
          "body": {
            "iso": "PJM",
            "node": "RTO",
            "unit": "USD/kWh",
            "lmp_usd_per_mwh": "49.24",
            "valid_on": "2026-09-10"
          }
        }"#,
    )
    .expect_err("unit");
    let msg = err.to_string();
    assert!(msg.contains("USD/MWh"), "{msg}");
}

#[test]
fn rejects_missing_lmp_token() {
    let err = ingest::lmp::parse_lmp_wrapper(
        br#"{
          "fetched_at": "2026-09-10T12:00:00.000Z",
          "source_url": "https://www.pjm.com/markets-and-operations.aspx",
          "body": {
            "iso": "PJM",
            "node": "RTO",
            "unit": "USD/MWh",
            "valid_on": "2026-09-10"
          }
        }"#,
    )
    .expect_err("missing");
    assert!(err.to_string().contains("lmp_usd_per_mwh"), "{err}");
}

#[test]
fn rejects_missing_valid_on() {
    let err = ingest::lmp::parse_lmp_wrapper(
        br#"{
          "fetched_at": "2026-09-10T12:00:00.000Z",
          "source_url": "https://www.pjm.com/markets-and-operations.aspx",
          "body": {
            "iso": "PJM",
            "node": "RTO",
            "unit": "USD/MWh",
            "lmp_usd_per_mwh": "49.24"
          }
        }"#,
    )
    .expect_err("missing valid_on");
    assert!(err.to_string().contains("valid_on"), "{err}");
}
