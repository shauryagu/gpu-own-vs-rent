use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// sha256 of fixtures/ocpi/current/H100_SXM.json — the hash events.jsonl names.
const H100_SHA: &str = "608fb5ff229d86b8bb4ac6f4af6170c48126f0dbdb47c11c774428cac455b95f";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn golden_catalog() -> &'static [u8] {
    include_bytes!("../../../fixtures/log/v1/catalog.json")
}

fn copy_v1() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = repo_root().join("fixtures/log/v1");
    fs::copy(src.join("events.jsonl"), tmp.path().join("events.jsonl")).expect("copy events");
    let cas_dst = tmp.path().join("cas");
    fs::create_dir_all(&cas_dst).expect("cas dir");
    for entry in fs::read_dir(src.join("cas")).expect("read cas") {
        let entry = entry.expect("cas entry");
        fs::copy(entry.path(), cas_dst.join(entry.file_name())).expect("copy cas blob");
    }
    tmp
}

fn replay(log_dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_chi"))
        .args(["replay", "--log-dir"])
        .arg(log_dir)
        .args(["--format", "json"])
        .output()
        .expect("chi replay")
}

fn assert_replay_ok(output: &Output) {
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn replay_v1_twice_stdout_matches_golden() {
    let tmp = copy_v1();
    let first = replay(tmp.path());
    let second = replay(tmp.path());
    assert_replay_ok(&first);
    assert_replay_ok(&second);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stdout, golden_catalog());
}

#[test]
fn decoy_timestamped_raw_is_ignored() {
    let tmp = copy_v1();
    let decoy = b"decoy-not-the-cas-blob";
    let fetched_at = "2026-08-22T21:03:21.660Z.json";
    for rel in [
        format!("raw/ocpi/current/H100_SXM/{fetched_at}"),
        format!("data/raw/ocpi/current/H100_SXM/{fetched_at}"),
    ] {
        let path = tmp.path().join(&rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("decoy dir");
        fs::write(&path, decoy).expect("write decoy");
    }

    let output = replay(tmp.path());
    assert_replay_ok(&output);
    assert_eq!(output.stdout, golden_catalog());
}

#[test]
fn missing_cas_exits_nonzero_without_golden_catalog() {
    let tmp = copy_v1();
    fs::remove_file(tmp.path().join("cas").join(H100_SHA)).expect("delete cas blob");

    let output = replay(tmp.path());
    assert!(
        !output.status.success(),
        "missing CAS must fail closed; stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_ne!(output.stdout.as_slice(), golden_catalog());
}

#[test]
fn mismatched_cas_exits_nonzero_without_golden_catalog() {
    let tmp = copy_v1();
    fs::write(tmp.path().join("cas").join(H100_SHA), b"wrong-bytes").expect("overwrite cas");

    let output = replay(tmp.path());
    assert!(
        !output.status.success(),
        "mismatched CAS must fail closed; stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_ne!(output.stdout.as_slice(), golden_catalog());
}

#[test]
fn unknown_event_type_exits_nonzero() {
    let tmp = copy_v1();
    fs::write(tmp.path().join("events.jsonl"), "{\"type\":\"Nope\"}\n").expect("write unknown tag");

    let output = replay(tmp.path());
    assert!(
        !output.status.success(),
        "unknown type must fail closed; stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn catalog_series_is_hourly_current_not_invert() {
    let tmp = copy_v1();
    let output = replay(tmp.path());
    assert_replay_ok(&output);

    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("catalog json");
    let entry = &value["entries"][0];
    assert_eq!(entry["series"], "ocpi.current");
    assert_eq!(entry["index_value"], "2.63");

    let text = String::from_utf8_lossy(&output.stdout);
    for forbidden in ["leftover", "implied_salvage", "fair_rent", "daily-index"] {
        assert!(
            !text.contains(forbidden),
            "catalog must not contain {forbidden}: {text}"
        );
    }
}

#[test]
fn replay_source_has_no_http_get_clock_or_invert_keys() {
    let src = include_str!("../src/replay.rs");
    for needle in [
        "HttpGet",
        "now(",
        "mtime",
        "leftover",
        "implied_salvage",
        "fair_rent",
    ] {
        assert!(
            !src.contains(needle),
            "crates/chi/src/replay.rs must not contain {needle}"
        );
    }
}
