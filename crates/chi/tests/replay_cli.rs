use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn chi() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_chi"));
    cmd.current_dir(repo_root());
    cmd
}

#[test]
fn chi_help_lists_replay_collect_and_invert() {
    let output = chi().arg("--help").output().expect("chi --help");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        stdout.contains("replay"),
        "expected replay in --help, got {stdout}"
    );
    assert!(
        stdout.contains("collect"),
        "expected collect in --help, got {stdout}"
    );
    assert!(
        stdout.contains("invert"),
        "expected invert in --help, got {stdout}"
    );
}

#[test]
fn invert_help_keeps_declared_flags_and_has_no_log_dir() {
    let output = chi()
        .args(["invert", "--help"])
        .output()
        .expect("chi invert --help");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    for flag in [
        "--purchase-cents",
        "--fixture-dir",
        "--life-years",
        "--utilization",
        "--discount-rate",
    ] {
        assert!(
            stdout.contains(flag),
            "invert --help missing {flag}: {stdout}"
        );
    }
    assert!(
        !stdout.contains("--log-dir"),
        "invert --help must not list --log-dir: {stdout}"
    );
}

#[test]
fn replay_help_has_log_dir() {
    let output = chi()
        .args(["replay", "--help"])
        .output()
        .expect("chi replay --help");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        stdout.contains("--log-dir"),
        "replay --help missing --log-dir: {stdout}"
    );
}

#[test]
fn replay_json_stdout_matches_v1_catalog_hourly_current_not_invert() {
    let output = chi()
        .args(["replay", "--log-dir", "fixtures/log/v1", "--format", "json"])
        .output()
        .expect("chi replay");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = output.stdout;
    let golden = include_bytes!("../../../fixtures/log/v1/catalog.json");
    assert_eq!(&stdout, golden);

    let text = String::from_utf8_lossy(&stdout);
    assert!(text.contains("ocpi.current"));
    assert!(text.contains("\"2.63\""));
    for forbidden in ["leftover", "implied_salvage", "fair_rent", "daily-index"] {
        assert!(
            !text.contains(forbidden),
            "catalog must not contain {forbidden}: {text}"
        );
    }
}
