use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};
use time::{OffsetDateTime, UtcOffset};

use crate::error::IngestError;

pub struct RawCache {
    data_dir: PathBuf,
    pause: Duration,
}

impl RawCache {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            pause: Duration::from_millis(200),
        }
    }

    pub fn with_pause(mut self, pause: Duration) -> Self {
        self.pause = pause;
        self
    }

    pub fn pause(&self) -> Duration {
        self.pause
    }

    pub fn write_gpu_types_free(
        &self,
        fetched_at: OffsetDateTime,
        bytes: &[u8],
    ) -> Result<PathBuf, IngestError> {
        let path = self
            .data_dir
            .join("raw/ocpi/gpu-types-free")
            .join(format!("{}.json", compact_fetched_at(fetched_at)));
        write_raw(&path, bytes)?;
        Ok(path)
    }

    pub fn write_current(
        &self,
        gpu_slug: &str,
        fetched_at: OffsetDateTime,
        bytes: &[u8],
    ) -> Result<PathBuf, IngestError> {
        let path = self
            .data_dir
            .join("raw/ocpi/current")
            .join(gpu_slug)
            .join(format!("{}.json", compact_fetched_at(fetched_at)));
        write_raw(&path, bytes)?;
        Ok(path)
    }

    pub fn append_hourly_line(&self, line: &str) -> Result<PathBuf, IngestError> {
        let path = self.data_dir.join("ocpi-hourly.jsonl");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(path)
    }
}

pub fn gpu_slug(name: &str) -> String {
    name.replace(' ', "_")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

pub fn compact_fetched_at(fetched_at: OffsetDateTime) -> String {
    let utc = fetched_at.to_offset(UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}{:02}{:02}{:09}Z",
        utc.year(),
        u8::from(utc.month()),
        utc.day(),
        utc.hour(),
        utc.minute(),
        utc.second(),
        utc.nanosecond(),
    )
}

pub fn json_fetched_at(fetched_at: OffsetDateTime) -> String {
    let utc = fetched_at.to_offset(UtcOffset::UTC);
    let millis = utc.nanosecond() / 1_000_000;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        utc.year(),
        u8::from(utc.month()),
        utc.day(),
        utc.hour(),
        utc.minute(),
        utc.second(),
        millis,
    )
}

fn write_raw(path: &Path, bytes: &[u8]) -> Result<(), IngestError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}
