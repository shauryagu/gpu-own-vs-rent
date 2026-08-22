use std::path::PathBuf;
use std::time::Duration;

use crate::error::IngestError;

pub struct RawResponse {
    pub status: u16,
    pub bytes: Vec<u8>,
    pub url: String,
}

pub trait HttpGet {
    fn get(&self, url: &str) -> Result<RawResponse, IngestError>;
}

pub struct LiveHttp {
    client: reqwest::blocking::Client,
}

impl LiveHttp {
    pub fn new() -> Result<Self, IngestError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("chi-collector/0.1 (research; OCPI current; no key)")
            .build()
            .map_err(IngestError::HttpClient)?;
        Ok(Self { client })
    }

    fn get_once(&self, url: &str) -> Result<RawResponse, IngestError> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|source| IngestError::Transport {
                url: url.to_string(),
                source,
            })?;
        let status = response.status().as_u16();
        let bytes = response
            .bytes()
            .map_err(|source| IngestError::Transport {
                url: url.to_string(),
                source,
            })?
            .to_vec();
        Ok(RawResponse {
            status,
            bytes,
            url: url.to_string(),
        })
    }
}

impl HttpGet for LiveHttp {
    fn get(&self, url: &str) -> Result<RawResponse, IngestError> {
        match self.get_once(url) {
            Ok(response) => Ok(response),
            Err(_) => self.get_once(url),
        }
    }
}

pub struct FixtureHttp {
    dir: PathBuf,
}

impl FixtureHttp {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
}

impl HttpGet for FixtureHttp {
    fn get(&self, url: &str) -> Result<RawResponse, IngestError> {
        let path = fixture_path(&self.dir, url);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(RawResponse {
                status: 200,
                bytes,
                url: url.to_string(),
            }),
            Err(_) => Ok(RawResponse {
                status: 404,
                bytes: Vec::new(),
                url: url.to_string(),
            }),
        }
    }
}

fn fixture_path(dir: &std::path::Path, url: &str) -> PathBuf {
    let url = url.split('?').next().unwrap_or(url);
    if url.ends_with("/api/gpu-types-free") {
        return dir.join("gpu-types-free.json");
    }
    const MARKER: &str = "/api/gpu/";
    if let Some(idx) = url.find(MARKER) {
        let rest = &url[idx + MARKER.len()..];
        if !rest.is_empty() && !rest.contains('/') {
            let name = percent_decode(rest);
            let slug = crate::cache::gpu_slug(&name);
            return dir.join("current").join(format!("{slug}.json"));
        }
    }
    dir.join("unknown.json")
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(value) = u8::from_str_radix(hex, 16) {
                    out.push(value);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
