use thiserror::Error;

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
}
