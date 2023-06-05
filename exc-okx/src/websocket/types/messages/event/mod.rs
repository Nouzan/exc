use crate::error::OkxError;

use self::ticker::OkxTicker;

use super::Args;
use exc_core::types::ticker::Ticker;
pub use instrument::OkxInstrumentMeta;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

mod instrument;
mod ticker;

/// Order.
pub mod order;

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

/// Okx order response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    /// Client order id.
    pub cl_ord_id: String,
    /// Order id.
    pub ord_id: String,
    /// Tag.
    pub tag: Option<String>,
    /// Code.
    pub s_code: String,
    /// Message.
    pub s_msg: String,
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

/// Okx websocket trade response type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum TradeResponse {
    /// Order.
    Order {
        /// Id.
        id: String,
        /// Code.
        code: String,
        /// Msg.
        msg: String,
        /// Data.
        data: Vec<OrderData>,
    },
    /// Order.
    CancelOrder {
        /// Id.
        id: String,
        /// Code.
        code: String,
        /// Msg.
        msg: String,
        /// Data.
        data: Vec<OrderData>,
    },
}

/// Action kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Action {
    /// A update change.
    #[default]
    Update,
    /// A snapsshot change.
    Snapshot,
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

impl Change {
    pub(crate) fn deserialize_data<T>(self) -> impl Iterator<Item = Result<T, serde_json::Error>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.data.into_iter().map(serde_json::from_value)
    }
}

/// Okx weboscket event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    /// Response.
    Response(ResponseKind),
    /// Change.
    Change(Change),
    /// Trade.
    TradeResponse(TradeResponse),
}

impl TryFrom<Event> for Vec<Result<Ticker, OkxError>> {
    type Error = OkxError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Change(change) => Ok(change
                .data
                .into_iter()
                .map(|v| {
                    serde_json::from_value::<OkxTicker>(v)
                        .map(Ticker::from)
                        .map_err(OkxError::from)
                })
                .collect()),
            Event::Response(resp) => Err(OkxError::UnexpectedDataType(anyhow::anyhow!(
                "response: {resp:?}"
            ))),
            Event::TradeResponse(resp) => Err(OkxError::UnexpectedDataType(anyhow::anyhow!(
                "response: {resp:?}"
            ))),
        }
    }
}
