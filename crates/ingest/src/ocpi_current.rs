use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use time::OffsetDateTime;

use crate::cache::{self, RawCache};
use crate::error::IngestError;
use crate::http::{HttpGet, RawResponse};

const GPU_TYPES_FREE_URL: &str = "https://api.ornnai.com/api/gpu-types-free";
const CURRENT_URL_PREFIX: &str = "https://api.ornnai.com/api/gpu/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentQuote {
    pub gpu_name: String,
    pub index_value: String,
    pub last_updated: String,
}

#[derive(Debug, Serialize)]
struct HourlyRecord<'a> {
    schema: &'static str,
    series: &'static str,
    gpu_name: &'a str,
    index_value: &'a str,
    source_last_updated: &'a str,
    fetched_at: String,
    valid_on: Option<&'a str>,
    source_url: &'a str,
    http_status: u16,
    raw_path: String,
    raw_sha256: String,
}

pub fn current_url(gpu_name: &str) -> String {
    format!("{CURRENT_URL_PREFIX}{}", percent_encode(gpu_name))
}

pub fn parse_gpu_types_free(bytes: &[u8]) -> Result<Vec<String>, IngestError> {
    let value: Value = serde_json::from_slice(bytes)?;
    if value.get("success") != Some(&Value::Bool(true)) {
        return Err(IngestError::Parse(
            "gpu-types-free: success is not true".to_string(),
        ));
    }
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| IngestError::Parse("gpu-types-free: missing data array".to_string()))?;
    let mut names = Vec::with_capacity(data.len());
    for item in data {
        let name = item
            .get("gpu_name")
            .and_then(Value::as_str)
            .ok_or_else(|| IngestError::Parse("gpu-types-free: missing gpu_name".to_string()))?;
        names.push(name.to_string());
    }
    Ok(names)
}

pub fn parse_current_body(bytes: &[u8]) -> Result<CurrentQuote, IngestError> {
    let value: Value = serde_json::from_slice(bytes)?;
    if value.get("success") != Some(&Value::Bool(true)) {
        return Err(IngestError::Parse(
            "ocpi current: success is not true".to_string(),
        ));
    }
    let data = value
        .get("data")
        .ok_or_else(|| IngestError::Parse("ocpi current: missing data".to_string()))?;
    let gpu_name = data
        .get("gpu_name")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("ocpi current: missing gpu_name".to_string()))?
        .to_string();
    let last_updated = data
        .get("last_updated")
        .and_then(Value::as_str)
        .ok_or_else(|| IngestError::Parse("ocpi current: missing last_updated".to_string()))?
        .to_string();
    let index_value = parse_index_value(
        data.get("index_value")
            .ok_or_else(|| IngestError::Parse("ocpi current: missing index_value".to_string()))?,
    )?;
    Ok(CurrentQuote {
        gpu_name,
        index_value,
        last_updated,
    })
}

/// Source token text via `from_str_exact`. Never transit `f64`.
pub fn parse_index_value(value: &Value) -> Result<String, IngestError> {
    let text = match value {
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        other => {
            return Err(IngestError::Parse(format!(
                "index_value must be a number or string, got {other}"
            )))
        }
    };
    let decimal = Decimal::from_str_exact(&text).map_err(|err| {
        IngestError::Parse(format!(
            "index_value {text:?} is not an exact decimal: {err}"
        ))
    })?;
    Ok(decimal.to_string())
}

pub fn collect_current(
    now: OffsetDateTime,
    http: &impl HttpGet,
    cache: &RawCache,
) -> Result<(), IngestError> {
    let types_response = http.get(GPU_TYPES_FREE_URL)?;
    let types_path = cache.write_gpu_types_free(now, &types_response.bytes)?;
    log_attempt(
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
    let names = parse_gpu_types_free(&types_response.bytes)?;
    if names.is_empty() {
        return Err(IngestError::EmptyGpuList);
    }

    let mut failures: Vec<String> = Vec::new();
    for (i, name) in names.iter().enumerate() {
        if i > 0 && !cache.pause().is_zero() {
            std::thread::sleep(cache.pause());
        }
        match collect_one_gpu(now, http, cache, name) {
            Ok(()) => {}
            Err(err) => failures.push(format!("{name}: {err}")),
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

fn collect_one_gpu(
    now: OffsetDateTime,
    http: &impl HttpGet,
    cache: &RawCache,
    gpu_name: &str,
) -> Result<(), IngestError> {
    let url = current_url(gpu_name);
    let slug = cache::gpu_slug(gpu_name);
    let response = match http.get(&url) {
        Ok(response) => response,
        Err(err) => {
            log_attempt(now, gpu_name, 0, 0, std::path::Path::new("-"), false);
            return Err(err);
        }
    };
    let raw_path = cache.write_current(&slug, now, &response.bytes)?;
    if response.status != 200 {
        log_attempt(
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
    let quote = match parse_current_body(&response.bytes) {
        Ok(quote) => quote,
        Err(err) => {
            log_attempt(
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
    if let Err(err) = append_hourly(now, cache, gpu_name, &quote, &response, &raw_path) {
        log_attempt(
            now,
            gpu_name,
            response.status,
            response.bytes.len(),
            &raw_path,
            false,
        );
        return Err(err);
    }
    log_attempt(
        now,
        gpu_name,
        response.status,
        response.bytes.len(),
        &raw_path,
        true,
    );
    Ok(())
}

fn append_hourly(
    now: OffsetDateTime,
    cache: &RawCache,
    gpu_name: &str,
    quote: &CurrentQuote,
    response: &RawResponse,
    raw_path: &std::path::Path,
) -> Result<(), IngestError> {
    let record = HourlyRecord {
        schema: "ocpi.hourly.v1",
        series: "ocpi.current",
        gpu_name,
        index_value: &quote.index_value,
        source_last_updated: &quote.last_updated,
        fetched_at: cache::json_fetched_at(now),
        valid_on: None,
        source_url: &response.url,
        http_status: response.status,
        raw_path: raw_path.display().to_string(),
        raw_sha256: cache::sha256_hex(&response.bytes),
    };
    let line = serde_json::to_string(&record)?;
    cache.append_hourly_line(&line)?;
    Ok(())
}

fn log_attempt(
    now: OffsetDateTime,
    gpu: &str,
    http_status: u16,
    bytes: usize,
    raw_path: &std::path::Path,
    ok: bool,
) {
    let status = if ok { "ok" } else { "err" };
    eprintln!(
        "fetched_at={} gpu={:?} http_status={} bytes={} raw_path={} {}",
        cache::json_fetched_at(now),
        gpu,
        http_status,
        bytes,
        raw_path.display(),
        status,
    );
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_bytes(rel: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(rel);
        std::fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
    }

    #[test]
    fn fixture_current_bodies_parse_to_five_names_and_string_prices() {
        let names = parse_gpu_types_free(&fixture_bytes("ocpi/gpu-types-free.json")).unwrap();
        assert_eq!(names, ["A100 SXM4", "B200", "H100 SXM", "H200", "RTX 5090"]);

        let expected = [
            ("A100 SXM4", "A100_SXM4"),
            ("B200", "B200"),
            ("H100 SXM", "H100_SXM"),
            ("H200", "H200"),
            ("RTX 5090", "RTX_5090"),
        ];
        for (name, slug) in expected {
            let quote =
                parse_current_body(&fixture_bytes(&format!("ocpi/current/{slug}.json"))).unwrap();
            assert_eq!(quote.gpu_name, name);
            assert!(
                quote
                    .index_value
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.'),
                "index_value should be decimal text, got {:?}",
                quote.index_value
            );
            Decimal::from_str_exact(&quote.index_value).unwrap();
        }
    }

    #[test]
    fn index_value_json_number_preserves_long_decimal_token() {
        let body = br#"{"success":true,"data":{"gpu_name":"H100 SXM","region":"","index_value":2.879583333333333,"last_updated":"2026-08-22T04:02:40.996Z"}}"#;
        let quote = parse_current_body(body).unwrap();
        assert_eq!(quote.index_value, "2.879583333333333");
        assert_ne!(quote.index_value, "2.8795833333333335");
        assert_ne!(quote.index_value, "2.88");
    }
}
