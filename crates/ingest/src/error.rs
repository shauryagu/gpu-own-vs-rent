use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("transport error fetching {url}: {source}")]
    Transport {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("http client: {0}")]
    HttpClient(#[source] reqwest::Error),
    #[error("http status {status} from {url}")]
    HttpStatus { url: String, status: u16 },
    #[error("parse error: {0}")]
    Parse(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("empty gpu-types-free list")]
    EmptyGpuList,
    #[error("collect failed for {count} gpu(s): {detail}")]
    CollectFailed { count: usize, detail: String },
}
