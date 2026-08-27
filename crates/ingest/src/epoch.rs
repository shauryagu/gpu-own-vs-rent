//! Epoch ML hardware CSV. Release price is an annotation, never purchase \(P\).

use std::str::FromStr;

use domain::GpuModel;
use rust_decimal::Decimal;

use crate::error::IngestError;

const H100_SXM_HARDWARE: &str = "NVIDIA H100 SXM5 80GB";
const H200_HARDWARE: &str = "NVIDIA H200 SXM";
const B200_HARDWARE: &str = "NVIDIA B200";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochRow {
    pub hardware_name: String,
    pub tdp_w: Decimal,
    pub memory_bytes: Option<Decimal>,
    pub tensor_fp16_flops: Option<Decimal>,
    pub fp8_flops: Option<Decimal>,
    pub release_date: Option<String>,
    pub release_price_usd: Option<Decimal>,
}

pub fn parse_ml_hardware_csv(bytes: &[u8]) -> Result<Vec<EpochRow>, IngestError> {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader
        .headers()
        .map_err(|err| IngestError::Parse(format!("epoch csv headers: {err}")))?
        .clone();
    let idx = |name: &str| {
        headers
            .iter()
            .position(|h| h == name)
            .ok_or_else(|| IngestError::Parse(format!("epoch csv missing column {name:?}")))
    };
    let hardware_i = idx("Hardware name")?;
    let tdp_i = idx("TDP (W)")?;
    let memory_i = idx("Memory (bytes)")?;
    let fp16_i = idx("Tensor-FP16/BF16 performance (FLOP/s)")?;
    let fp8_i = idx("FP8 performance (FLOP/s)")?;
    let date_i = idx("Release date")?;
    let price_i = idx("Release price (USD)")?;

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|err| IngestError::Parse(format!("epoch csv row: {err}")))?;
        let hardware_name = record
            .get(hardware_i)
            .ok_or_else(|| IngestError::Parse("epoch csv: missing hardware name".into()))?
            .trim()
            .to_string();
        if hardware_name.is_empty() {
            continue;
        }
        let Some(tdp_w) = optional_decimal(record.get(tdp_i).unwrap_or(""))? else {
            continue;
        };
        rows.push(EpochRow {
            hardware_name,
            tdp_w,
            memory_bytes: optional_decimal(record.get(memory_i).unwrap_or(""))?,
            tensor_fp16_flops: optional_decimal(record.get(fp16_i).unwrap_or(""))?,
            fp8_flops: optional_decimal(record.get(fp8_i).unwrap_or(""))?,
            release_date: optional_text(record.get(date_i).unwrap_or("")),
            release_price_usd: optional_decimal(record.get(price_i).unwrap_or(""))?,
        });
    }
    Ok(rows)
}

/// Invert mapping. H100 SXM is the only Gate 1 invert row.
pub fn mapped_hardware_name(gpu: GpuModel) -> Option<&'static str> {
    match gpu {
        GpuModel::H100Sxm => Some(H100_SXM_HARDWARE),
        GpuModel::H200 => Some(H200_HARDWARE),
        GpuModel::B200 => Some(B200_HARDWARE),
        GpuModel::A100Sxm4 | GpuModel::Rtx5090 => None,
    }
}

pub fn row_for_gpu<'a>(
    gpu: GpuModel,
    rows: &'a [EpochRow],
    energy_requested: bool,
) -> Result<&'a EpochRow, IngestError> {
    match mapped_hardware_name(gpu) {
        Some(name) => rows
            .iter()
            .find(|row| row.hardware_name == name)
            .ok_or_else(|| IngestError::Parse(format!("epoch csv has no row {name}"))),
        None if energy_requested => Err(IngestError::UnmappedGpuEnergy {
            gpu: format!("{gpu:?}"),
        }),
        None => Err(IngestError::UnmappedEpochGpu {
            gpu: format!("{gpu:?}"),
        }),
    }
}

fn optional_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn optional_decimal(raw: &str) -> Result<Option<Decimal>, IngestError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Decimal::from_str_exact(trimmed)
        .or_else(|_| Decimal::from_str(trimmed))
        .map(Some)
        .map_err(|err| IngestError::Parse(format!("epoch decimal {trimmed:?}: {err}")))
}

pub const ML_HARDWARE_URL: &str = "https://epoch.ai/data/ml_hardware.csv";

pub fn collect_epoch(
    now: time::OffsetDateTime,
    http: &impl crate::http::HttpGet,
    cache: &crate::cache::RawCache,
) -> Result<(), IngestError> {
    let _lock = cache.try_lock()?;
    let response = http.get(ML_HARDWARE_URL)?;
    let path = cache.write_epoch_ml_hardware(now, &response.bytes)?;
    crate::ocpi_current::log_attempt(
        now,
        "epoch.ml_hardware",
        response.status,
        response.bytes.len(),
        &path,
        response.status == 200,
    );
    if response.status != 200 {
        return Err(IngestError::HttpStatus {
            url: response.url,
            status: response.status,
        });
    }
    let rows = parse_ml_hardware_csv(&response.bytes)?;
    if rows.is_empty() {
        return Err(IngestError::Parse("epoch csv has no hardware rows".into()));
    }
    Ok(())
}
