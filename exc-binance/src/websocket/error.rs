use thiserror::Error;
use tower::BoxError;

use super::protocol::frame::Name;

/// Websocket API errors.
#[derive(Debug, Error)]
pub enum WsError {
    /// Errors from tokio-tower.
    #[error("tokio-tower: {0}")]
    TokioTower(anyhow::Error),
    /// Transport timeout.
    #[error("transport timeout")]
    TransportTimeout,
    /// Transport is borken.
    #[error("transport is broken")]
    TransportIsBoken,
    /// Websocket errors.
    #[error("websocket: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    /// Json errors.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Duplicate stream id.
    #[error("duplicate stream id")]
    DuplicateStreamId,
    /// No response.
    #[error("no response")]
    NoResponse,
    /// Unexpected frame.
    #[error("unexpected frame: {0}")]
    UnexpectedFrame(anyhow::Error),
    /// Stream has been subscribed.
    #[error("stream {0} has been subscribed")]
    StreamSubscribed(Name),
    /// Empty stream name.
    #[error("empty stream name")]
    EmptyStreamName,
    /// Unknown connection error.
    #[error("unknown connection error: {0}")]
    UnknownConnection(BoxError),
    /// Main stream not found.
    #[error("main stream not found")]
    MainStreamNotFound,
}
