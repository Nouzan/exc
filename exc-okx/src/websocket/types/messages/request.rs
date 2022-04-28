use super::Args;
use crate::error::OkxError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use tokio_tungstenite::tungstenite::Message;

/// Okx websocket operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

impl fmt::Display for WsRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subscribe(args) => {
                write!(f, "sub:{args}")
            }
            Self::Unsubscribe(args) => {
                write!(f, "unsub:{args}")
            }
        }
    }
}

impl WsRequest {
    /// Subscribe tickers.
    pub fn subscribe_tickers(inst: &str) -> Self {
        Self::Subscribe(Args(BTreeMap::from([
            ("channel".to_string(), "tickers".to_string()),
            ("instId".to_string(), inst.to_string()),
        ])))
    }

    /// Unsubscribe tickers.
    pub fn unsubscribe_tickers(inst: &str) -> Self {
        Self::Unsubscribe(Args(BTreeMap::from([
            ("channel".to_string(), "tickers".to_string()),
            ("instId".to_string(), inst.to_string()),
        ])))
    }
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
