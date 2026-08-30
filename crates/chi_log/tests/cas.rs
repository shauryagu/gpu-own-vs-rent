use std::fs;
use std::path::Path;

/// sha256("hello")
const HELLO_SHA: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

fn write_file(dir: &Path, rel: &str, body: &[u8]) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

#[test]
fn matching_cas_blob_returns_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{HELLO_SHA}"), b"hello");
    let bytes = chi_log::open_cas(tmp.path(), HELLO_SHA).expect("matching blob");
    assert_eq!(bytes, b"hello");
}

#[test]
fn wrong_cas_bytes_are_err() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{HELLO_SHA}"), b"world");
    chi_log::open_cas(tmp.path(), HELLO_SHA).expect_err("hash must match key");
}

#[test]
fn missing_cas_file_is_err() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("cas")).unwrap();
    chi_log::open_cas(tmp.path(), HELLO_SHA).expect_err("missing blob");
}

#[test]
fn decoy_timestamped_raw_path_is_not_read() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{HELLO_SHA}"), b"hello");
    write_file(
        tmp.path(),
        "data/raw/ocpi/current/H100_SXM/2026-08-22T210321660000000Z.json",
        b"world",
    );
    write_file(
        tmp.path(),
        "raw/ocpi/current/H100_SXM/2026-08-22T210321660000000Z.json",
        b"world",
    );
    let bytes = chi_log::open_cas(tmp.path(), HELLO_SHA).expect("cas wins over decoy");
    assert_eq!(bytes, b"hello");
}

#[test]
fn missing_cas_is_err_even_when_decoy_raw_exists() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(
        tmp.path(),
        "data/raw/ocpi/current/H100_SXM/2026-08-22T210321660000000Z.json",
        b"hello",
    );
    chi_log::open_cas(tmp.path(), HELLO_SHA).expect_err("must not glob fetched_at or data/raw");
}
