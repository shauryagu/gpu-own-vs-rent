//! Daily-index invert \(S\) and daily-history (refused as \(S\)).

use domain::{FetchedAt, GpuModel, ObservedSpot, SpotSeries, UsdPerGpuHour, ValidOn};
use rust_decimal::Decimal;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::IngestError;
use crate::ocpi_current::parse_index_value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyIndexRecord {
    pub fetched_at: FetchedAt,
    pub source_url: String,
    pub spot: ObservedSpot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryPoint {
    pub timestamp: String,
    pub index_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRecord {
    pub gpu_type: String,
    pub points: Vec<HistoryPoint>,
}

pub fn parse_daily_index_wrapper(bytes: &[u8]) -> Result<DailyIndexRecord, IngestError> {
    let value: Value = serde_json::from_slice(bytes)?;
    let fetched_at = parse_fetched_at(
        value
            .get("fetched_at")
            .and_then(Value::as_str)
            .ok_or_else(|| IngestError::Parse("wrapper missing fetched_at".to_string()))?,
    )?;
    let source_url = value
        .get("source_url")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("wrapper missing source_url".to_string()))?
        .to_string();
    let body = value
        .get("body")
        .ok_or_else(|| IngestError::Parse("wrapper missing body".to_string()))?;
    let spot = parse_daily_index_body(body)?;
    Ok(DailyIndexRecord {
        fetched_at,
        source_url,
        spot,
    })
}

fn daily_index_data(value: &Value) -> Result<&Value, IngestError> {
    if value.get("success") != Some(&Value::Bool(true)) {
        return Err(IngestError::Parse(
            "daily-index: success is not true".to_string(),
        ));
    }
    let data = value
        .get("data")
        .ok_or_else(|| IngestError::Parse("daily-index: missing data".to_string()))?;
    if data.is_array() {
        return Err(IngestError::Parse(
            "daily-index: data is an array; invert S is per-GPU only".to_string(),
        ));
    }
    Ok(data)
}

fn daily_index_gpu_type(value: &Value) -> Result<&str, IngestError> {
    daily_index_data(value)?
        .get("gpu_type")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("daily-index: missing gpu_type".to_string()))
}

pub fn parse_daily_index_body(value: &Value) -> Result<ObservedSpot, IngestError> {
    let gpu_type = daily_index_gpu_type(value)?;
    let gpu = gpu_model_from_ocpi_name(gpu_type)
        .ok_or_else(|| IngestError::Parse(format!("daily-index: unknown gpu_type {gpu_type:?}")))?;
    let data = daily_index_data(value)?;
    let date = data
        .get("date")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("daily-index: missing date".to_string()))?;
    let valid_on = valid_on_utc(date)?;
    let token = parse_index_value(
        data.get("index_value")
            .ok_or_else(|| IngestError::Parse("daily-index: missing index_value".to_string()))?,
    )?;
    let decimal = Decimal::from_str_exact(&token).map_err(|err| {
        IngestError::Parse(format!(
            "daily-index index_value {token:?} is not an exact decimal: {err}"
        ))
    })?;
    let price = UsdPerGpuHour::try_from(decimal)
        .map_err(|err| IngestError::Parse(format!("daily-index price: {err}")))?;
    Ok(ObservedSpot {
        gpu,
        series: SpotSeries::OcpiDailyIndex,
        valid_on,
        price,
    })
}

pub fn parse_daily_history(bytes: &[u8]) -> Result<HistoryRecord, IngestError> {
    let value: Value = serde_json::from_slice(bytes)?;
    if value.get("success") != Some(&Value::Bool(true)) {
        return Err(IngestError::Parse(
            "daily-history: success is not true".to_string(),
        ));
    }
    let gpu_type = value
        .get("gpu_type")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("daily-history: missing gpu_type".to_string()))?
        .to_string();
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| IngestError::Parse("daily-history: missing data array".to_string()))?;
    let mut points = Vec::with_capacity(data.len());
    for item in data {
        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .ok_or_else(|| IngestError::Parse("daily-history: missing timestamp".to_string()))?
            .to_string();
        let index_value = parse_index_value(item.get("index_value").ok_or_else(|| {
            IngestError::Parse("daily-history: missing index_value".to_string())
        })?)?;
        points.push(HistoryPoint {
            timestamp,
            index_value,
        });
    }
    Ok(HistoryRecord { gpu_type, points })
}

pub fn gpu_model_from_ocpi_name(name: &str) -> Option<GpuModel> {
    match name {
        "H100 SXM" => Some(GpuModel::H100Sxm),
        "H200" => Some(GpuModel::H200),
        "B200" => Some(GpuModel::B200),
        "A100 SXM4" => Some(GpuModel::A100Sxm4),
        "RTX 5090" => Some(GpuModel::Rtx5090),
        _ => None,
    }
}

fn parse_fetched_at(text: &str) -> Result<FetchedAt, IngestError> {
    let when = OffsetDateTime::parse(text, &Rfc3339)
        .map_err(|err| IngestError::Parse(format!("wrapper fetched_at {text:?}: {err}")))?;
    Ok(FetchedAt::new(when))
}

fn valid_on_utc(text: &str) -> Result<ValidOn, IngestError> {
    let when = OffsetDateTime::parse(text, &Rfc3339)
        .map_err(|err| IngestError::Parse(format!("daily-index date {text:?}: {err}")))?;
    Ok(ValidOn::new(when.to_offset(time::UtcOffset::UTC).date()))
}

pub const DAILY_INDEX_ALL_URL: &str = "https://api.ornnai.com/api/daily-index/all";

pub fn daily_index_url(gpu_name: &str) -> String {
    format!(
        "https://api.ornnai.com/api/daily-index?gpu={}",
        crate::ocpi_current::percent_encode(gpu_name)
    )
}

pub fn daily_history_url(gpu_name: &str) -> String {
    format!(
        "https://api.ornnai.com/api/gpu/{}/index-history",
        crate::ocpi_current::percent_encode(gpu_name)
    )
}

pub fn collect_daily(
    now: OffsetDateTime,
    http: &impl crate::http::HttpGet,
    cache: &crate::cache::RawCache,
) -> Result<(), IngestError> {
    let _lock = cache.try_lock()?;
    let mut failures: Vec<String> = Vec::new();

    let types_response = http.get(crate::ocpi_current::GPU_TYPES_FREE_URL)?;
    let types_path = cache.write_gpu_types_free(now, &types_response.bytes)?;
    crate::ocpi_current::log_attempt(
        now,
        "gpu-types-free",
        types_response.status,
        types_response.bytes.len(),
        &types_path,
        types_response.status == 200,
    );
    if types_response.status != 200 {
        return Err(IngestError::HttpStatus {
            url: types_response.url,
            status: types_response.status,
        });
    }
    let names = crate::ocpi_current::parse_gpu_types_free(&types_response.bytes)?;
    if names.is_empty() {
        return Err(IngestError::EmptyGpuList);
    }

    sleep_pause(cache);
    let all_response = http.get(DAILY_INDEX_ALL_URL)?;
    let all_path = cache.write_daily_index_all(now, &all_response.bytes)?;
    let all_ok = all_response.status == 200;
    crate::ocpi_current::log_attempt(
        now,
        "daily-index-all",
        all_response.status,
        all_response.bytes.len(),
        &all_path,
        all_ok,
    );
    if !all_ok {
        failures.push(format!("daily-index-all: http {}", all_response.status));
    }

    for name in &names {
        sleep_pause(cache);
        if let Err(err) = collect_one_daily_index(now, http, cache, name) {
            failures.push(format!("{name} daily-index: {err}"));
        }
        sleep_pause(cache);
        if let Err(err) = collect_one_history(now, http, cache, name) {
            failures.push(format!("{name} daily-history: {err}"));
        }
    }

    if !failures.is_empty() {
        return Err(IngestError::CollectFailed {
            count: failures.len(),
            detail: failures.join("; "),
        });
    }
    Ok(())
}

fn collect_one_daily_index(
    now: OffsetDateTime,
    http: &impl crate::http::HttpGet,
    cache: &crate::cache::RawCache,
    gpu_name: &str,
) -> Result<(), IngestError> {
    let url = daily_index_url(gpu_name);
    let slug = crate::cache::gpu_slug(gpu_name);
    let response = http.get(&url)?;
    let raw_path = cache.write_daily_index(&slug, now, &response.bytes)?;
    let ok_http = response.status == 200;
    if !ok_http {
        crate::ocpi_current::log_attempt(
            now,
            gpu_name,
            response.status,
            response.bytes.len(),
            &raw_path,
            false,
        );
        return Err(IngestError::HttpStatus {
            url: response.url,
            status: response.status,
        });
    }
    let value: Value = serde_json::from_slice(&response.bytes)?;
    let got = match daily_index_gpu_type(&value) {
        Ok(got) => got,
        Err(err) => {
            crate::ocpi_current::log_attempt(
                now,
                gpu_name,
                response.status,
                response.bytes.len(),
                &raw_path,
                false,
            );
            return Err(err);
        }
    };
    if got != gpu_name {
        crate::ocpi_current::log_attempt(
            now,
            gpu_name,
            response.status,
            response.bytes.len(),
            &raw_path,
            false,
        );
        return Err(IngestError::GpuTypeMismatch {
            expected: gpu_name.to_string(),
            got: got.to_string(),
        });
    }
    crate::ocpi_current::log_attempt(
        now,
        gpu_name,
        response.status,
        response.bytes.len(),
        &raw_path,
        true,
    );
    Ok(())
}

fn collect_one_history(
    now: OffsetDateTime,
    http: &impl crate::http::HttpGet,
    cache: &crate::cache::RawCache,
    gpu_name: &str,
) -> Result<(), IngestError> {
    let url = daily_history_url(gpu_name);
    let slug = crate::cache::gpu_slug(gpu_name);
    let response = http.get(&url)?;
    let raw_path = cache.write_daily_history(&slug, now, &response.bytes)?;
    if response.status != 200 {
        crate::ocpi_current::log_attempt(
            now,
            gpu_name,
            response.status,
            response.bytes.len(),
            &raw_path,
            false,
        );
        return Err(IngestError::HttpStatus {
            url: response.url,
            status: response.status,
        });
    }
    match parse_daily_history(&response.bytes) {
        Ok(history) if history.gpu_type == gpu_name => {
            crate::ocpi_current::log_attempt(
                now,
                gpu_name,
                response.status,
                response.bytes.len(),
                &raw_path,
                true,
            );
            Ok(())
        }
        Ok(history) => {
            crate::ocpi_current::log_attempt(
                now,
                gpu_name,
                response.status,
                response.bytes.len(),
                &raw_path,
                false,
            );
            Err(IngestError::GpuTypeMismatch {
                expected: gpu_name.to_string(),
                got: history.gpu_type,
            })
        }
        Err(err) => {
            crate::ocpi_current::log_attempt(
                now,
                gpu_name,
                response.status,
                response.bytes.len(),
                &raw_path,
                false,
            );
            Err(err)
        }
    }
}

fn sleep_pause(cache: &crate::cache::RawCache) {
    if !cache.pause().is_zero() {
        std::thread::sleep(cache.pause());
    }
}
