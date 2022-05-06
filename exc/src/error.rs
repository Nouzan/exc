use thiserror::Error;

/// Exchange Errors.
#[derive(Debug, Error)]
pub enum ExchangeError {
    /// Error from layers.
    #[error(transparent)]
    Layer(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[cfg(feature = "http")]
    /// Http errors.
    #[error("http: {0}")]
    Http(hyper::Error),
    /// All other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
