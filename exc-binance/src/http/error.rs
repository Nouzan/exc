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
    /// Query string errors.
    #[error("qs: {0}")]
    Qs(#[from] serde_qs::Error),
    /// Standard exchange errors.
    #[error("exchange: {0}")]
    Exchange(#[from] ExchangeError),
    /// Unexpected response type.
    #[error("unexpected response type: {0}")]
    UnexpectedResponseType(anyhow::Error),
    /// Unsupported endpoint.
    #[error("unsuppored endpoint: {0}")]
    UnsupportedEndpoint(anyhow::Error),
}

impl RestError {
    /// Is temp.
    pub fn is_temporary(&self) -> bool {
        if let Self::Exchange(err) = self {
            err.is_temporary()
        } else {
            false
        }
    }
}
