use thiserror::Error;

use crate::{http::error::RestError, websocket::error::WsError};

/// All errors in [`exc-binance`]
#[derive(Debug, Error)]
pub enum BinanceError {
    /// Rest API errors.
    #[error("rest: {0}")]
    Rest(#[from] RestError),
    /// Websocket API errors.
    #[error("websocket: {0}")]
    Ws(#[from] WsError),
    /// All other errors.
    #[error("unknown: {0}")]
    Unknown(#[from] anyhow::Error),
}
