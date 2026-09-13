//! Named LMP snapshot (USD/MWh) → π (USD/kWh). No HTTP.

use domain::{FetchedAt, UsdPerKwh, ValidOn};
use rust_decimal::Decimal;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};

use crate::error::IngestError;
use crate::ocpi_current::parse_index_value;

/// Frozen public LMP print. `lmp_usd_per_mwh` is the source token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LmpRecord {
    pub fetched_at: FetchedAt,
    pub source_url: String,
    pub iso: String,
    pub node: String,
    pub valid_on: ValidOn,
    pub lmp_usd_per_mwh: String,
    pub usd_per_kwh: UsdPerKwh,
}

pub fn parse_lmp_wrapper(bytes: &[u8]) -> Result<LmpRecord, IngestError> {
    let value: Value = serde_json::from_slice(bytes)?;
    let fetched_at = parse_fetched_at(
        value
            .get("fetched_at")
            .and_then(Value::as_str)
            .ok_or_else(|| IngestError::Parse("lmp wrapper missing fetched_at".to_string()))?,
    )?;
    let source_url = value
        .get("source_url")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("lmp wrapper missing source_url".to_string()))?
        .to_string();
    let body = value
        .get("body")
        .ok_or_else(|| IngestError::Parse("lmp wrapper missing body".to_string()))?;

    let iso = body
        .get("iso")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("lmp body missing iso".to_string()))?
        .to_string();
    let node = body
        .get("node")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("lmp body missing node".to_string()))?
        .to_string();
    let unit = body
        .get("unit")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("lmp body missing unit".to_string()))?;
    if unit != "USD/MWh" {
        return Err(IngestError::Parse(format!(
            "lmp unit {unit:?} is not USD/MWh"
        )));
    }
    let valid_on = parse_valid_on(
        body.get("valid_on")
            .and_then(Value::as_str)
            .ok_or_else(|| IngestError::Parse("lmp body missing valid_on".to_string()))?,
    )?;
    let token = parse_index_value(
        body.get("lmp_usd_per_mwh")
            .ok_or_else(|| IngestError::Parse("lmp body missing lmp_usd_per_mwh".to_string()))?,
    )?;
    let usd_per_kwh = usd_per_kwh_from_mwh(&token)?;

    Ok(LmpRecord {
        fetched_at,
        source_url,
        iso,
        node,
        valid_on,
        lmp_usd_per_mwh: token,
        usd_per_kwh,
    })
}

/// LMP is USD/MWh. π is USD/kWh. Divide by 1000; keep Decimal (no f64).
pub fn usd_per_kwh_from_mwh(token: &str) -> Result<UsdPerKwh, IngestError> {
    let mwh = Decimal::from_str_exact(token).map_err(|err| {
        IngestError::Parse(format!(
            "lmp_usd_per_mwh {token:?} is not an exact decimal: {err}"
        ))
    })?;
    let kwh = mwh / Decimal::from(1000);
    UsdPerKwh::try_from(kwh).map_err(|err| IngestError::Parse(format!("usd per kWh: {err}")))
}

fn parse_fetched_at(text: &str) -> Result<FetchedAt, IngestError> {
    let when = OffsetDateTime::parse(text, &Rfc3339)
        .map_err(|err| IngestError::Parse(format!("lmp fetched_at {text:?}: {err}")))?;
    Ok(FetchedAt::new(when))
}

fn parse_valid_on(text: &str) -> Result<ValidOn, IngestError> {
    let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
        .expect("valid_on date format");
    let day = Date::parse(text, &format)
        .map_err(|err| IngestError::Parse(format!("lmp valid_on {text:?}: {err}")))?;
    Ok(ValidOn::new(day))
}
