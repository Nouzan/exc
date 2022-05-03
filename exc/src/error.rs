use thiserror::Error;

/// Exchange Errors.
#[derive(Debug, Error)]
pub enum ExchangeError {
    #[cfg(feature = "http")]
    /// Http errors.
    #[error("http: {0}")]
    Http(hyper::Error),
    /// All other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
