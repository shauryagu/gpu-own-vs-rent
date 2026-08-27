use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn invert() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_chi"));
    cmd.current_dir(repo_root());
    cmd.args([
        "invert",
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
    ]);
    cmd
}

#[test]
fn invert_text_prints_both_inverses_omits_forward_and_matches_golden() {
    let output = invert()
        .args(["--format", "text"])
        .output()
        .expect("run chi invert");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("ocpi.daily-index"));
    assert!(stdout.contains("L leftover"));
    assert!(stdout.contains("R* salvage"));
    assert!(stdout.contains("Hourly current was not used."));
    assert!(stdout.contains("Daily history was not used"));
    assert!(stdout.contains("Half-life was not estimated."));
    assert!(!stdout.contains("implied residual"));
    assert!(!stdout.contains("\nForward\n"));
    let golden = include_str!("invert_fixture.stdout.txt");
    assert_eq!(stdout, golden);
}

#[test]
fn invert_json_keeps_source_token_and_named_inverses() {
    let output = invert()
        .args(["--format", "json"])
        .output()
        .expect("run chi invert json");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["spot"]["series"], "ocpi.daily-index");
    assert_eq!(v["spot"]["s_usd_per_gpu_hour"], "2.879583333333333");
    assert!(v["forward"].is_null());
    assert!(v["inverse"]["leftover_usd_per_gpu_hour"].is_string());
    assert!(v["inverse"]["implied_salvage_usd"].is_string());
    assert!(v.get("implied_residual").is_none());
    assert!(v["inverse"].get("implied_residual").is_none());
}

#[test]
fn invert_a100_is_fail_closed() {
    let output = Command::new(env!("CARGO_BIN_EXE_chi"))
        .current_dir(repo_root())
        .args([
            "invert",
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
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    let err = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        err.contains("A100") || err.contains("Unmapped") || err.contains("mapping"),
        "expected fail-closed A100, got {err}"
    );
}

#[test]
fn invert_missing_daily_index_names_the_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_chi"))
        .args([
            "invert",
            "--gpu",
            "H100 SXM",
            "--fixture-dir",
            "/no/such/chi-fixtures",
            "--purchase-cents",
            "2500000",
            "--life-years",
            "5",
            "--utilization",
            "0.60",
            "--discount-rate",
            "0.10",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        err.contains("ocpi.daily-index"),
        "missing fixture must name ocpi.daily-index, got {err}"
    );
}
