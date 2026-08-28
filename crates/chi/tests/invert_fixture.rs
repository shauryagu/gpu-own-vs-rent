use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn invert() -> Command {
    invert_gpu("H100 SXM")
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
fn invert_json_matches_binary_golden_at_twelve_dp() {
    let output = invert()
        .args(["--format", "json"])
        .output()
        .expect("run chi invert json");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(v["spot"]["series"], "ocpi.daily-index");
    assert_eq!(v["spot"]["s_usd_per_gpu_hour"], "2.879583333333333");
    assert_eq!(
        v["declared"]["capital_rent_usd_per_gpu_hour"],
        "1.254744486276"
    );
    assert_eq!(v["inverse"]["leftover_usd_per_gpu_hour"], "1.624838847057");
    assert_eq!(v["inverse"]["implied_salvage_usd"], "-52138.487958999989");
    assert_eq!(v["accounting"]["f_usd_per_gpu_hour"], "1.092120340386");
    assert_eq!(
        v["accounting"]["implied_salvage_usd"],
        "-72487.426754899986"
    );
    assert!(v["forward"].is_null());
    assert!(v["declared"]["salvage_usd"].is_null());
    assert!(v.get("implied_residual").is_none());
    assert!(v["inverse"].get("implied_residual").is_none());
    let golden = include_str!("invert_fixture.json");
    assert_eq!(stdout, golden);
}

#[test]
fn invert_residual_cents_zero_prints_forward_not_omitted() {
    let output = invert()
        .args(["--residual-cents", "0", "--format", "text"])
        .output()
        .expect("run chi invert residual 0");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("\nForward\n"));
    assert!(stdout.contains("F(θ)"));
    assert!(stdout.contains("L leftover"));
    assert!(stdout.contains("R* salvage"));
    assert!(!stdout.contains("implied residual"));
    assert!(!stdout.contains("[no salvage declared]"));
    assert!(!stdout.contains("F(θ) omitted"));
}

#[test]
fn invert_residual_cents_zero_json_declares_salvage() {
    let output = invert()
        .args(["--residual-cents", "0", "--format", "json"])
        .output()
        .expect("run chi invert residual 0 json");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["declared"]["salvage_usd"], "0.00");
    assert!(v["forward"].is_object());
    assert_eq!(v["inverse"]["leftover_usd_per_gpu_hour"], "1.624838847057");
    assert_eq!(v["inverse"]["implied_salvage_usd"], "-52138.487958999989");
    assert!(v.get("implied_residual").is_none());
}

fn invert_gpu(gpu: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_chi"));
    cmd.current_dir(repo_root());
    cmd.args([
        "invert",
        "--gpu",
        gpu,
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

fn stderr_and_stdout(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    )
}

fn assert_unmapped_gpu_not_missing_fixture(gpu: &str) {
    let output = invert_gpu(gpu).output().expect("run");
    assert!(!output.status.success());
    let err = stderr_and_stdout(&output);
    assert!(
        err.contains("unmapped GPU"),
        "expected Epoch fail-closed, got {err}"
    );
    assert!(
        !err.contains("missing ocpi.daily-index"),
        "must reach Epoch, not die on a missing wrapper: {err}"
    );
}

#[test]
fn invert_a100_is_fail_closed_at_epoch() {
    assert_unmapped_gpu_not_missing_fixture("A100 SXM4");
}

#[test]
fn invert_rtx_5090_is_fail_closed_at_epoch() {
    assert_unmapped_gpu_not_missing_fixture("RTX 5090");
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
