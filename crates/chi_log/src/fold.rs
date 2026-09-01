use std::path::Path;

use serde::Serialize;

use crate::cas::open_cas;
use crate::error::Error;
use crate::Event;

/// Ingest catalog: fetches and parse outcomes. Not invert stdout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Catalog {
    pub entries: Vec<CatalogEntry>,
}

/// One fetch, optionally filled by a later `SeriesParsed` with the same hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CatalogEntry {
    pub series: String,
    pub gpu_name: Option<String>,
    pub fetched_at: String,
    pub valid_on: Option<String>,
    pub raw_sha256: String,
    pub index_value: Option<String>,
    /// Always serialized. Null until a parse-failure event exists.
    pub parse_error: Option<String>,
}

/// Project log-order fetches. A missing CAS blob fails the fold.
pub fn fold(events: &[Event], log_dir: &Path) -> Result<Catalog, Error> {
    let mut entries = Vec::new();
    for event in events {
        match event {
            Event::SourceFetched(fetched) => {
                open_cas(log_dir, &fetched.raw_sha256)?;
                entries.push(CatalogEntry {
                    series: fetched.series.clone(),
                    gpu_name: None,
                    fetched_at: fetched.fetched_at.clone(),
                    valid_on: None,
                    raw_sha256: fetched.raw_sha256.clone(),
                    index_value: None,
                    parse_error: None,
                });
            }
            Event::SeriesParsed(parsed) => {
                if let Some(entry) = entries.iter_mut().rev().find(|entry| {
                    entry.raw_sha256 == parsed.raw_sha256
                        && entry.gpu_name.is_none()
                        && entry.index_value.is_none()
                }) {
                    entry.gpu_name = Some(parsed.gpu_name.clone());
                    entry.index_value = Some(parsed.index_value.clone());
                    entry.valid_on = parsed.valid_on.clone();
                }
            }
        }
    }
    Ok(Catalog { entries })
}

/// Compact JSON so a second fold can compare bytes to the golden catalog.
pub fn catalog_bytes(catalog: &Catalog) -> Result<Vec<u8>, Error> {
    Ok(serde_json::to_vec(catalog).expect("catalog is strings and options"))
}
