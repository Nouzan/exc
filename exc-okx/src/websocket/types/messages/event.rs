use super::Args;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Message with code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMessage {
    /// Code.
    pub code: String,
    /// Message.
    pub msg: String,
}

impl fmt::Display for CodeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code={}, msg={}", self.code, self.msg)
    }
}

/// Okx websocket response type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum ResponseKind {
    /// Login success response.
    Login(CodeMessage),
    /// Subscribed response.
    Subscribe {
        /// Arg.
        arg: Args,
    },
    /// Unsubscribed response.
    Unsubscribe {
        /// Arg.
        arg: Args,
    },
    /// Error response.
    Error(CodeMessage),
}

/// Action kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// A update change.
    Update,
    /// A snapsshot change.
    Snapshot,
}

impl Default for Action {
    fn default() -> Self {
        Action::Update
    }
}

/// Change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Argument.
    pub arg: Args,

    /// Action.
    #[serde(default)]
    pub action: Action,

    /// Data.
    pub data: Vec<Value>,
}

/// Okx weboscket event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    /// Response.
    Response(ResponseKind),
    /// Change.
    Change(Change),
}
