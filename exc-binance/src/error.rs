use anyhow::anyhow;
use exc_core::ExchangeError;
use thiserror::Error;

use crate::{http::error::RestError, websocket::error::WsError};

/// All errors in [`exc-binance`]
#[derive(Debug, Error)]
pub enum Error {
    /// Rest API errors.
    #[error("rest: {0}")]
    Rest(#[from] RestError),
    /// Websocket API errors.
    #[error("websocket: {0}")]
    Ws(#[from] WsError),
    /// All other errors.
    #[error("unknown: {0}")]
    Unknown(#[from] anyhow::Error),
    /// Wrong response type.
    #[error("wrong response type")]
    WrongResponseType,
}

impl From<Error> for ExchangeError {
    fn from(err: Error) -> Self {
        match err {
            Error::Unknown(err) => Self::Other(err),
            Error::WrongResponseType => Self::Other(anyhow!("wrong response type")),
            Error::Rest(err) => match err {
                RestError::Http(_) | RestError::Hyper(_) => Self::Unavailable(err.into()),
                RestError::Exchange(err) => err,
                _ => Self::Other(err.into()),
            },
            Error::Ws(err) => match &err {
                WsError::ListenKeyExpired(_)
                | WsError::StreamSubscribed(_)
                | WsError::TokioTower(_)
                | WsError::TransportIsBoken
                | WsError::TransportTimeout
                | WsError::UnknownConnection(_)
                | WsError::Websocket(_) => Self::Unavailable(err.into()),
                _ => Self::Other(err.into()),
            },
        }
    }
}
