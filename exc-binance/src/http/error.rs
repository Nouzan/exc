use exc_core::ExchangeError;
use thiserror::Error;

/// Rest API Errors.
#[derive(Debug, Error)]
pub enum RestError {
    /// API error message.
    #[error("api: code={0} msg={0}")]
    Api(i64, String),
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
    /// Urlencoded.
    #[error("urlencoded: {0}")]
    Urlencoded(#[from] serde_urlencoded::ser::Error),
    /// Standard exchange errors.
    #[error("exchange: {0}")]
    Exchange(#[from] ExchangeError),
    /// Invalid header value.
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    /// Unexpected response type.
    #[error("unexpected response type: {0}")]
    UnexpectedResponseType(anyhow::Error),
    /// Unsupported endpoint.
    #[error("unsuppored endpoint: {0}")]
    UnsupportedEndpoint(anyhow::Error),
    /// Need key.
    #[error("need apikey to sign the params")]
    NeedApikey,
    /// Sign error.
    #[error("sign error: {0}")]
    SignError(#[from] crate::types::key::SignError),
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
