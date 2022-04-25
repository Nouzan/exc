use super::Args;
use serde::{Deserialize, Serialize};

/// Okx websocket event type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum EventKind {
    /// Subscribe.
    Subscribe,
    /// Unsubscribe.
    Unsubscribe,
    /// Error.
    Error,
}

/// Okx weboscket event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Id.
    #[serde(default)]
    pub id: Option<String>,

    /// Event.
    pub event: EventKind,

    /// Args.
    #[serde(default)]
    pub args: Vec<Args>,

    /// Code.
    #[serde(default)]
    pub code: Option<String>,

    /// Message.
    #[serde(default)]
    pub message: String,
}
