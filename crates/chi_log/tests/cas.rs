use std::fs;
use std::path::Path;

use chi_log::Error;

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
fn wrong_cas_bytes_are_cas_hash_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    write_file(tmp.path(), &format!("cas/{HELLO_SHA}"), b"world");
    match chi_log::open_cas(tmp.path(), HELLO_SHA) {
        Err(Error::CasHashMismatch { raw_sha256, actual }) => {
            assert_eq!(raw_sha256, HELLO_SHA);
            assert_ne!(actual, HELLO_SHA);
            assert_eq!(actual.len(), 64);
        }
        other => panic!("expected CasHashMismatch, got {other:?}"),
    }
}

#[test]
fn missing_cas_file_is_missing_cas() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("cas")).unwrap();
    match chi_log::open_cas(tmp.path(), HELLO_SHA) {
        Err(Error::MissingCas { raw_sha256 }) => assert_eq!(raw_sha256, HELLO_SHA),
        other => panic!("expected MissingCas, got {other:?}"),
    }
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
    match chi_log::open_cas(tmp.path(), HELLO_SHA) {
        Err(Error::MissingCas { raw_sha256 }) => assert_eq!(raw_sha256, HELLO_SHA),
        other => panic!("expected MissingCas, not decoy raw, got {other:?}"),
    }
}

fn assert_invalid_cas_key(key: &str) {
    let tmp = tempfile::tempdir().unwrap();
    match chi_log::open_cas(tmp.path(), key) {
        Err(Error::InvalidCasKey(got)) => assert_eq!(got, key),
        other => panic!("expected InvalidCasKey for {key:?}, got {other:?}"),
    }
}

#[test]
fn uppercase_hex_is_invalid_cas_key() {
    assert_invalid_cas_key(&HELLO_SHA.to_ascii_uppercase());
}

#[test]
fn short_key_is_invalid_cas_key() {
    assert_invalid_cas_key("abc");
}

#[test]
fn path_traversal_key_is_invalid_cas_key() {
    let key = format!("../{}", "a".repeat(61));
    assert_eq!(key.len(), 64);
    assert_invalid_cas_key(&key);
}

#[test]
fn extra_path_segment_is_invalid_cas_key() {
    assert_invalid_cas_key(&format!("{HELLO_SHA}/x"));
}
