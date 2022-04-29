use thiserror::Error;

use crate::websocket::types::messages::{request::WsRequest, Args};

/// All OKX errors.
#[derive(Debug, Error)]
pub enum OkxError {
    /// Websocket error.
    #[error("weboscket: {0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    /// Remote closed.
    #[error("remote closed")]
    RemoteClosed,
    /// Connection error.
    #[error("connection error: {0}")]
    Connection(Box<dyn std::error::Error + Send + Sync>),
    /// Websocket disconnected.
    #[error("websocket disconnected")]
    WebsocketDisconnected,
    /// Ping error.
    #[error("ping error: {0}")]
    Ping(anyhow::Error),
    /// Ping timeout.
    #[error("ping timeout")]
    PingTimeout,
    /// Json error.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Request sender dropped.
    #[error("request sender dropped")]
    RequestSenderDropped,
    /// Dispatch error.
    #[error("dispatch error: req={0:?}")]
    Dispatch(WsRequest),
    /// Callback error.
    #[error("responser error: {0}")]
    Callback(#[from] tokio::sync::oneshot::error::RecvError),
    /// Already subscribed or unsubscribing.
    #[error("already subscribed or unsubscribping {0:?}")]
    SubscribedOrUnsubscribing(Args),
    /// Subscribing or unsubscribing.
    #[error("subscribing or unsubscribing {0:?}")]
    SubscribingOrUnsubscribing(Args),
    /// Websocket closed.
    #[error("websocket closed")]
    WebsocketClosed,
    /// API Error.
    #[error("api error: {0}")]
    Api(String),
    /// Protocol Error.
    #[error("protocol: {0}")]
    Protocol(anyhow::Error),
    /// Layers error.
    #[error(transparent)]
    Layer(Box<dyn std::error::Error + Send + Sync>),
    /// Buffer Layer Error.
    #[error(transparent)]
    Buffer(Box<dyn std::error::Error + Send + Sync>),
}
