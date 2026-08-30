use thiserror::Error;

/// Failures opening CAS blobs or reading the JSONL log.
#[derive(Debug, Error)]
pub enum Error {
    #[error("cas key must be 64 lowercase hex characters, got {0:?}")]
    InvalidCasKey(String),
    #[error("cas blob not found for {raw_sha256}")]
    MissingCas { raw_sha256: String },
    #[error("cas blob {raw_sha256} does not match file bytes (got {actual})")]
    CasHashMismatch { raw_sha256: String, actual: String },
    #[error("invalid jsonl line {line}: {source}")]
    InvalidLine {
        line: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
