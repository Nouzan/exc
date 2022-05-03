use thiserror::Error;

/// Exchange Errors.
#[derive(Debug, Error)]
pub enum ExchangeError {
    /// All other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
