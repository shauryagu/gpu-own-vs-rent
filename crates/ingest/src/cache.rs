use std::fs::{self, File, OpenOptions};
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

    /// Exclusive lock for the lifetime of one collect. Kernel-released on crash.
    pub fn try_lock(&self) -> Result<CollectLock, IngestError> {
        CollectLock::acquire(&self.data_dir)
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
        file.sync_all()?;
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

pub struct CollectLock {
    _file: File,
}

impl CollectLock {
    fn acquire(data_dir: &Path) -> Result<Self, IngestError> {
        fs::create_dir_all(data_dir)?;
        let path = data_dir.join("collect.lock");
        let file = OpenOptions::new().create(true).write(true).open(&path)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(std::fs::TryLockError::WouldBlock) => Err(IngestError::AlreadyRunning),
            Err(std::fs::TryLockError::Error(err)) => Err(err.into()),
        }
    }
}

fn write_raw(path: &Path, bytes: &[u8]) -> Result<(), IngestError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_name = match path.file_name() {
        Some(name) => {
            let mut tmp = name.to_os_string();
            tmp.push(".tmp");
            tmp
        }
        None => {
            return Err(IngestError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "raw path has no file name",
            )));
        }
    };
    let tmp_path = path.with_file_name(tmp_name);
    fs::write(&tmp_path, bytes)?;
    File::open(&tmp_path)?.sync_all()?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}
