use std::path::PathBuf;

use domain::GpuModel;
use ingest::epoch::{parse_ml_hardware_csv, row_for_gpu};
use ingest::error::IngestError;
use rust_decimal::Decimal;

fn excerpt_bytes() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/epoch/ml_hardware.excerpt.csv");
    std::fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

#[test]
fn epoch_excerpt_maps_h100_sxm_to_sxm5_80gb_tdp_700() {
    let rows = parse_ml_hardware_csv(&excerpt_bytes()).expect("csv");
    let row = row_for_gpu(GpuModel::H100Sxm, &rows, true).expect("mapped");
    assert_eq!(row.hardware_name, "NVIDIA H100 SXM5 80GB");
    assert_eq!(row.tdp_w, Decimal::from(700));
    assert_eq!(row.release_price_usd, Some(Decimal::from(33600)));
}

#[test]
fn epoch_release_price_is_annotation_only() {
    let rows = parse_ml_hardware_csv(&excerpt_bytes()).expect("csv");
    let row = row_for_gpu(GpuModel::H100Sxm, &rows, false).expect("mapped");
    assert_eq!(row.release_price_usd, Some(Decimal::from(33600)));
}

#[test]
fn epoch_skips_empty_tdp_and_accepts_scientific_flops() {
    let csv = b"Hardware name,TDP (W),Memory (bytes),Tensor-FP16/BF16 performance (FLOP/s),FP8 performance (FLOP/s),Release date,Release price (USD)\n\
No TDP,,,,,,\n\
Has Sci,700.0,,,1e+16,2022-09-20,33600.0\n";
    let rows = parse_ml_hardware_csv(csv).expect("csv");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].hardware_name, "Has Sci");
    assert_eq!(rows[0].tdp_w, Decimal::from(700));
    assert!(rows[0].fp8_flops.is_some());
}

#[test]
fn rtx_5090_with_energy_is_err() {
    let rows = parse_ml_hardware_csv(&excerpt_bytes()).expect("csv");
    let err = row_for_gpu(GpuModel::Rtx5090, &rows, true).unwrap_err();
    assert!(matches!(err, IngestError::UnmappedGpuEnergy { .. }));
}

#[test]
fn a100_sxm4_is_fail_closed() {
    let rows = parse_ml_hardware_csv(&excerpt_bytes()).expect("csv");
    let err = row_for_gpu(GpuModel::A100Sxm4, &rows, true).unwrap_err();
    assert!(matches!(err, IngestError::UnmappedGpuEnergy { .. }));
}
