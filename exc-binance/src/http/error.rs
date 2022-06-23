use exc::ExchangeError;
use thiserror::Error;

/// Rest API Errors.
#[derive(Debug, Error)]
pub enum RestError {
    /// Http errors.
    #[error("http: {0}")]
    Http(#[from] http::Error),
    /// Errors from hyper.
    #[error("hyper: {0}")]
    Hyper(#[from] hyper::Error),
    /// Json errors.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Standard exchange errors.
    #[error("exchange: {0}")]
    Exchange(#[from] ExchangeError),
}
