use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn stack() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_chi"));
    cmd.current_dir(repo_root());
    cmd.args([
        "stack",
        "--gpu",
        "H100 SXM",
        "--fixture-dir",
        "fixtures",
        "--purchase-cents",
        "2500000",
        "--life-years",
        "5",
        "--utilization",
        "0.60",
        "--discount-rate",
        "0.10",
        "--lmp-fixture",
        "fixtures/energy/lmp/pjm_rto.json",
    ]);
    cmd
}

fn stderr_and_stdout(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    )
}

#[test]
fn stack_twice_is_byte_identical() {
    let a = stack()
        .args(["--format", "text"])
        .output()
        .expect("stack run 1");
    let b = stack()
        .args(["--format", "text"])
        .output()
        .expect("stack run 2");
    assert!(
        a.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&a.stderr)
    );
    assert!(b.status.success());
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn stack_lmp_fixture_prints_token_49_24() {
    let output = stack()
        .args(["--format", "text"])
        .output()
        .expect("stack lmp");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        stdout.contains("49.24"),
        "expected LMP token 49.24, got {stdout}"
    );
    assert!(stdout.contains("ocpi.daily-index") || stdout.contains("daily-index"));
    assert!(stdout.contains("L leftover"));
    assert!(!stdout.contains("implied_salvage"));
    assert!(!stdout.contains("R* salvage"));
    assert!(!stdout.contains("implied residual"));
}

#[test]
fn stack_json_has_leftover_key_not_salvage() {
    let output = stack()
        .args(["--format", "json"])
        .output()
        .expect("stack json");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["spot"]["series"], "ocpi.daily-index");
    assert_eq!(v["energy"]["lmp_usd_per_mwh"], "49.24");
    let rows = v["rows"].as_array().expect("rows");
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row.get("leftover_usd_per_gpu_hour").is_some());
        assert!(row.get("implied_salvage_usd").is_none());
    }
    assert!(v.get("implied_salvage").is_none());
    assert!(v.get("implied_residual").is_none());
}

#[test]
fn stack_default_three_pue_rows_in_order() {
    let output = stack()
        .args(["--format", "json"])
        .output()
        .expect("stack default pue");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let rows = v["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["pue"], 1.0);
    assert_eq!(rows[1]["pue"], 1.2);
    assert_eq!(rows[2]["pue"], 1.5);
    // capital equal; leftover falls as PUE rises
    assert_eq!(
        rows[0]["capital_rent_usd_per_gpu_hour"],
        rows[1]["capital_rent_usd_per_gpu_hour"]
    );
    assert_eq!(
        rows[1]["capital_rent_usd_per_gpu_hour"],
        rows[2]["capital_rent_usd_per_gpu_hour"]
    );
    let l0 = rust_decimal::Decimal::from_str_exact(
        rows[0]["leftover_usd_per_gpu_hour"].as_str().expect("L0"),
    )
    .expect("L0 dec");
    let l1 = rust_decimal::Decimal::from_str_exact(
        rows[1]["leftover_usd_per_gpu_hour"].as_str().expect("L1"),
    )
    .expect("L1 dec");
    let l2 = rust_decimal::Decimal::from_str_exact(
        rows[2]["leftover_usd_per_gpu_hour"].as_str().expect("L2"),
    )
    .expect("L2 dec");
    assert!(
        l0 > l1 && l1 > l2,
        "leftover must fall with PUE: {l0} {l1} {l2}"
    );
}

#[test]
fn stack_single_pue_one_row() {
    let output = stack()
        .args(["--format", "json", "--pue", "1.0"])
        .output()
        .expect("stack --pue 1.0");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let rows = v["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["pue"], 1.0);
}

#[test]
fn stack_a100_is_fail_closed_at_epoch() {
    let output = Command::new(env!("CARGO_BIN_EXE_chi"))
        .current_dir(repo_root())
        .args([
            "stack",
            "--gpu",
            "A100 SXM4",
            "--fixture-dir",
            "fixtures",
            "--purchase-cents",
            "2500000",
            "--life-years",
            "5",
            "--utilization",
            "0.60",
            "--discount-rate",
            "0.10",
            "--lmp-fixture",
            "fixtures/energy/lmp/pjm_rto.json",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    let err = stderr_and_stdout(&output);
    assert!(
        err.contains("unmapped GPU"),
        "expected Epoch fail-closed, got {err}"
    );
    assert!(
        !err.contains("missing ocpi.daily-index"),
        "must reach Epoch, not die on missing wrapper: {err}"
    );
}

#[test]
fn stack_manual_energy_named_source_works() {
    let output = Command::new(env!("CARGO_BIN_EXE_chi"))
        .current_dir(repo_root())
        .args([
            "stack",
            "--gpu",
            "H100 SXM",
            "--fixture-dir",
            "fixtures",
            "--purchase-cents",
            "2500000",
            "--life-years",
            "5",
            "--utilization",
            "0.60",
            "--discount-rate",
            "0.10",
            "--energy-usd-per-kwh",
            "0.10",
            "--energy-source",
            "documented constant",
            "--format",
            "json",
            "--pue",
            "1.0",
        ])
        .output()
        .expect("manual energy");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["energy"]["source"], "documented constant");
    assert!(v["energy"]["lmp_usd_per_mwh"].is_null());
    assert_eq!(v["rows"].as_array().expect("rows").len(), 1);
}
