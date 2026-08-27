use std::path::PathBuf;

use domain::{GpuModel, SpotSeries};
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
fn wrapper_parses_to_daily_index_record_with_exact_token() {
    let record = ingest::ocpi_daily::parse_daily_index_wrapper(&fixture_bytes(
        "ocpi/daily-index/H100_SXM.json",
    ))
    .expect("wrapper");

    let fetched = OffsetDateTime::new_in_offset(
        Date::from_calendar_date(2026, Month::August, 22).unwrap(),
        Time::from_hms_nano(12, 0, 0, 0).unwrap(),
        UtcOffset::UTC,
    );
    assert_eq!(record.fetched_at.get(), fetched);
    assert_eq!(
        record.source_url,
        "https://api.ornnai.com/api/daily-index?gpu=H100%20SXM"
    );
    assert_eq!(record.spot.gpu, GpuModel::H100Sxm);
    assert_eq!(record.spot.series, SpotSeries::OcpiDailyIndex);
    assert_eq!(record.spot.price.amount(), dec("2.879583333333333"));
    assert_ne!(record.spot.price.amount(), dec("2.88"));
}

#[test]
fn inner_only_json_is_err() {
    let inner = br#"{
        "success": true,
        "data": {
            "gpu_type": "H100 SXM",
            "region": "global",
            "index_value": 2.879583333333333,
            "date": "2026-08-21T20:00:00.000Z"
        }
    }"#;
    let err = ingest::ocpi_daily::parse_daily_index_wrapper(inner).unwrap_err();
    assert!(
        err.to_string().contains("wrapper") || err.to_string().contains("fetched_at"),
        "inner-only must be Err, got {err}"
    );
}

#[test]
fn extra_fields_do_not_fail_parse() {
    let body = br#"{
      "fetched_at": "2026-08-22T12:00:00.000Z",
      "source_url": "https://api.ornnai.com/api/daily-index?gpu=H100%20SXM",
      "body": {
        "success": true,
        "count": 1,
        "data": {
          "gpu_type": "H100 SXM",
          "region": "global",
          "access": "public",
          "count": 1,
          "index_value": 2.879583333333333,
          "date": "2026-08-21T20:00:00.000Z"
        }
      }
    }"#;
    let record = ingest::ocpi_daily::parse_daily_index_wrapper(body).expect("extras");
    assert_eq!(record.spot.price.amount(), dec("2.879583333333333"));
}

#[test]
fn valid_on_is_utc_date_of_body_data_date() {
    let record = ingest::ocpi_daily::parse_daily_index_wrapper(&fixture_bytes(
        "ocpi/daily-index/H100_SXM.json",
    ))
    .expect("wrapper");
    assert_eq!(
        record.spot.valid_on.get(),
        Date::from_calendar_date(2026, Month::August, 21).unwrap()
    );
}

#[test]
fn gpu_name_instead_of_gpu_type_is_err() {
    let body = br#"{
      "fetched_at": "2026-08-22T12:00:00.000Z",
      "source_url": "https://api.ornnai.com/api/daily-index?gpu=H100%20SXM",
      "body": {
        "success": true,
        "data": {
          "gpu_name": "H100 SXM",
          "region": "global",
          "index_value": 2.879583333333333,
          "date": "2026-08-21T20:00:00.000Z"
        }
      }
    }"#;
    assert!(ingest::ocpi_daily::parse_daily_index_wrapper(body).is_err());
}

#[test]
fn daily_index_all_fixture_is_not_invert_s() {
    let value: serde_json::Value =
        serde_json::from_slice(&fixture_bytes("ocpi/daily-index-all.json")).expect("json");
    let err = ingest::ocpi_daily::parse_daily_index_body(&value).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("array") || msg.contains("per-GPU"),
        "all-GPU envelope must not parse as invert S, got {err}"
    );
}

#[test]
fn history_bytes_are_not_invert_s() {
    let value: serde_json::Value =
        serde_json::from_slice(&fixture_bytes("ocpi/daily-history/H100_SXM.json")).expect("json");
    assert!(ingest::ocpi_daily::parse_daily_index_body(&value).is_err());
}

#[test]
fn history_last_point_is_two_decimal_and_not_invert_s() {
    let history =
        ingest::ocpi_daily::parse_daily_history(&fixture_bytes("ocpi/daily-history/H100_SXM.json"))
            .expect("history");
    let last = history.points.last().expect("non-empty");
    assert_eq!(last.index_value, "2.88");

    let invert = ingest::ocpi_daily::parse_daily_index_wrapper(&fixture_bytes(
        "ocpi/daily-index/H100_SXM.json",
    ))
    .expect("invert S");
    assert_ne!(dec(&last.index_value), invert.spot.price.amount());
}
