use thiserror::Error;

#[derive(Debug, Error)]
pub enum JupiterError {
    #[error("failed to call Jupiter API: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse response body: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("release asset matching host not found; host triple candidates: {0}")]
    AssetNotFound(String),
    #[error("process is already running")]
    AlreadyRunning,
    #[error("no process is running")]
    #[allow(dead_code)]
    NotRunning,
    #[error("download failed for {url}: {source}")]
    DownloadFailed {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("download returned status {status} for {url}")]
    DownloadStatus {
        url: String,
        status: reqwest::StatusCode,
    },
    #[error("archive extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("health check failed: {0}")]
    HealthCheck(String),
    #[error("API request to {endpoint} failed with status {status}: {body}")]
    ApiStatus {
        endpoint: String,
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("unexpected response schema: {0}")]
    Schema(String),
    #[error("failed to construct IP-bound HTTP client: {0}")]
    ClientPool(String),
}
