use std::convert::Infallible;

use thiserror::Error;

/// Instrument Errors.
#[derive(Debug, Error)]
pub enum InstrumentError {
    /// Instrument does not exist.
    #[error("instrument does not exist")]
    NotFound,
}

/// Exchange Errors.
#[derive(Debug, Error)]
pub enum ExchangeError {
    /// Error from layers.
    #[error("layer: {0}")]
    Layer(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[cfg(feature = "http")]
    /// Http errors.
    #[error("http: {0}")]
    Http(hyper::Error),
    /// All other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    /// All other api errors.
    #[error("api: {0}")]
    Api(anyhow::Error),
    /// Unavailable.
    #[error("unavailable: {0}")]
    Unavailable(anyhow::Error),
    /// Instrument errors.
    #[error("instrument: {0}")]
    Instrument(InstrumentError),
    /// Rate limited.
    #[error("rate limited: {0}")]
    RateLimited(anyhow::Error),
    /// API Key error.
    #[error("key error: {0}")]
    KeyError(anyhow::Error),
    /// Order not found.
    #[error("order not found")]
    OrderNotFound,
    /// Forbidden.
    #[error("forbidden: {0}")]
    Forbidden(anyhow::Error),
    /// Unexpected response type.
    #[error("unexpected response type: {0}")]
    UnexpectedResponseType(String),
}

impl ExchangeError {
    /// Is temporary.
    pub fn is_temporary(&self) -> bool {
        #[cfg(feature = "http")]
        {
            matches!(
                self,
                Self::RateLimited(_) | Self::Unavailable(_) | Self::Http(_)
            )
        }
        #[cfg(not(feature = "http"))]
        {
            matches!(self, Self::RateLimited(_) | Self::Unavailable(_))
        }
    }

    /// Flatten.
    pub fn flatten(self) -> Self {
        match self {
            Self::Layer(err) => match err.downcast::<Self>() {
                Ok(err) => (*err).flatten(),
                Err(err) => Self::Other(anyhow::anyhow!("{err}")),
            },
            err => err,
        }
    }

    /// Flatten layered error.
    pub fn layer(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        match err.downcast::<Self>() {
            Ok(err) => (*err).flatten(),
            Err(err) => Self::Other(anyhow::anyhow!("{err}")),
        }
    }

    /// Unexpected response type.
    pub fn unexpected_response_type(msg: impl ToString) -> Self {
        Self::UnexpectedResponseType(msg.to_string())
    }
}

impl From<Infallible> for ExchangeError {
    fn from(_: Infallible) -> Self {
        panic!("infallible")
    }
}
