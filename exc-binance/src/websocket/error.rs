use thiserror::Error;
use tower::BoxError;

use crate::http::error::RestError;

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
    /// Remote close.
    #[error("websocket: remote close (received a close frame)")]
    RemoteClose,
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
    /// Invalid Uri.
    #[error("invalid uri")]
    InvalidUri(#[from] http::uri::InvalidUri),
    /// Login error.
    #[error("login: {0}")]
    Login(#[from] RestError),
    /// Listen key is expired.
    #[error("listen key is expired: at={0}")]
    ListenKeyExpired(i64),
}
