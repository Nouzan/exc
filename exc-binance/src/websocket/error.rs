use thiserror::Error;

use super::{protocol::Protocol, request::WsRequest};

/// Websocket API errors.
#[derive(Debug, Error)]
pub enum WsError {
    /// Errors from tokio-tower.
    #[error("tokio-tower: {0}")]
    TokioTower(anyhow::Error),
    /// Transport timeout.
    #[error("transport timeout")]
    TransportTimeout,
    /// Websocket errors.
    #[error("websocket: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
}

impl From<tokio_tower::Error<Protocol, WsRequest>> for WsError {
    fn from(err: tokio_tower::Error<Protocol, WsRequest>) -> Self {
        match err {
            tokio_tower::Error::BrokenTransportSend(err)
            | tokio_tower::Error::BrokenTransportRecv(Some(err)) => err,
            err => Self::TokioTower(err.into()),
        }
    }
}
