use super::Args;
use crate::error::OkxError;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

/// Okx websocket operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum Op {
    /// Subscribe.
    Subscribe,
    /// Unsubsribe.
    Unsubscribe,
    // /// Login.
    // Login,
}

/// Okx websocket request messagee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequestMessage {
    /// Id.
    #[serde(default)]
    pub id: Option<String>,
    /// Operation.
    pub op: Op,
    /// Arguments.
    #[serde(default)]
    pub args: Vec<Args>,
}

impl WsRequestMessage {
    /// Convert into a websocket message.
    pub fn to_websocket(&self) -> Result<Message, OkxError> {
        let text = serde_json::to_string(&self)?;
        Ok(Message::Text(text))
    }
}

/// Okx websocket request.
#[derive(Debug, Clone)]
pub enum WsRequest {
    /// Subscribe.
    Subscribe(Args),
    /// Unsubscribe.
    Unsubscribe(Args),
}

impl Into<WsRequestMessage> for WsRequest {
    fn into(self) -> WsRequestMessage {
        match self {
            Self::Subscribe(args) => WsRequestMessage {
                id: None,
                op: Op::Subscribe,
                args: vec![args],
            },
            Self::Unsubscribe(args) => WsRequestMessage {
                id: None,
                op: Op::Unsubscribe,
                args: vec![args],
            },
        }
    }
}
