use thiserror::Error;

/// All OKX errors.
#[derive(Debug, Error)]
pub enum OkxError {
    /// Websocket error.
    #[error("weboscket: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    /// Connection error.
    #[error("connection error: {0}")]
    Connection(Box<dyn std::error::Error + Send + Sync>),
}
