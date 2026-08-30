//! Append-only ingest events. Replay folds these; invert does not read them.

mod cas;
mod error;
mod jsonl;

pub use cas::open_cas;
pub use error::Error;
pub use jsonl::read_events;

use serde::{Deserialize, Serialize};

/// One append-only ingest event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    SourceFetched(SourceFetched),
    SeriesParsed(SeriesParsed),
}

/// HTTP fetch of a named series body, keyed by content hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFetched {
    pub fetched_at: String,
    pub source_url: String,
    pub http_status: u16,
    pub series: String,
    pub raw_sha256: String,
}

/// Parse of a fetched body. `valid_on` is null for `ocpi.current`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesParsed {
    pub series: String,
    pub gpu_name: String,
    pub index_value: String,
    pub valid_on: Option<String>,
    pub raw_sha256: String,
}
