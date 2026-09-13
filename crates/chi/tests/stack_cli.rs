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

fn stack_base() -> Command {
    let mut cmd = chi();
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
    ]);
    cmd
}

#[test]
fn chi_help_lists_stack_and_about_lines_for_all_commands() {
    let output = chi().arg("--help").output().expect("chi --help");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    for cmd in ["collect", "invert", "replay", "stack"] {
        assert!(
            stdout.contains(cmd),
            "expected {cmd} in chi --help, got {stdout}"
        );
    }
    // clap prints variant about text next to each subcommand
    assert!(
        stdout.contains("Collect free public OCPI and Epoch snapshots"),
        "collect about missing: {stdout}"
    );
    assert!(
        stdout.contains("Teaching inverse: leftover L and salvage R*"),
        "invert about missing: {stdout}"
    );
    assert!(
        stdout.contains("Fold a chi_log directory into a catalog"),
        "replay about missing: {stdout}"
    );
    assert!(
        stdout.contains("Cost-stack panel: S = F_capital + e + L"),
        "stack about missing: {stdout}"
    );
}

#[test]
fn stack_missing_energy_identity_is_non_zero_exit() {
    let output = stack_base().output().expect("run stack without energy");
    assert!(
        !output.status.success(),
        "stack without named π must fail closed"
    );
    let err = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        err.contains("energy") || err.contains("lmp") || err.contains("π"),
        "error should name missing energy identity, got {err}"
    );
}

#[test]
fn stack_energy_usd_without_source_fails() {
    let output = stack_base()
        .args(["--energy-usd-per-kwh", "0.10"])
        .output()
        .expect("run");
    assert!(!output.status.success());
}

#[test]
fn stack_energy_source_without_usd_fails() {
    let output = stack_base()
        .args(["--energy-source", "manual"])
        .output()
        .expect("run");
    assert!(!output.status.success());
}

#[test]
fn stack_rs_has_no_http_now_or_leftover_salvage_collapse() {
    let src = include_str!("../src/stack.rs");
    for needle in [
        "HttpGet",
        "now(",
        "OffsetDateTime::now",
        "implied_residual",
        "implied residual",
        "implied_salvage",
    ] {
        assert!(!src.contains(needle), "stack.rs must not contain {needle}");
    }
    // leftover and salvage must not be collapsed under one name
    assert!(
        !src.to_lowercase().contains("leftover salvage")
            && !src.to_lowercase().contains("salvage leftover"),
        "must not collapse leftover/salvage"
    );
}
