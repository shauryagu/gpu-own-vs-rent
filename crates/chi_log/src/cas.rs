use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::Error;

/// Open `log_dir/cas/{raw_sha256}` and require the bytes to hash to that key.
///
/// The key is the path component. This does not search `fetched_at` or `data/raw`.
pub fn open_cas(log_dir: &Path, raw_sha256: &str) -> Result<Vec<u8>, Error> {
    if !is_lowercase_sha256_hex(raw_sha256) {
        return Err(Error::InvalidCasKey(raw_sha256.to_string()));
    }
    let path = log_dir.join("cas").join(raw_sha256);
    let bytes = fs::read(&path).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => Error::MissingCas {
            raw_sha256: raw_sha256.to_string(),
        },
        _ => Error::Io(err),
    })?;
    let actual = sha256_hex(&bytes);
    if actual != raw_sha256 {
        return Err(Error::CasHashMismatch {
            raw_sha256: raw_sha256.to_string(),
            actual,
        });
    }
    Ok(bytes)
}

fn is_lowercase_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}
