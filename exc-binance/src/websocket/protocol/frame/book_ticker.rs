use rust_decimal::Decimal;
use serde::Deserialize;

use crate::websocket::error::WsError;

use super::{Name, Nameable, StreamFrame, StreamFrameKind};

/// Book ticker.
#[derive(Debug, Clone, Deserialize)]
pub struct BookTicker {
    /// Event type.
    #[serde(rename = "e")]
    pub event: String,
    /// Event time.
    #[serde(rename = "E")]
    pub event_timestamp: i64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Trade time.
    #[serde(rename = "T")]
    pub trade_timestamp: i64,
    /// Book ticker ID.
    #[serde(rename = "u")]
    pub id: usize,
    /// Best bid.
    #[serde(rename = "b")]
    pub bid: Decimal,
    /// Best bid size.
    #[serde(rename = "B")]
    pub bid_size: Decimal,
    /// Best bid.
    #[serde(rename = "a")]
    pub ask: Decimal,
    /// Best bid size.
    #[serde(rename = "A")]
    pub ask_size: Decimal,
}

impl Nameable for BookTicker {
    fn to_name(&self) -> Name {
        Name {
            inst: Some(self.symbol.to_lowercase()),
            channel: self.event.clone(),
        }
    }
}

impl TryFrom<StreamFrame> for BookTicker {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::BookTicker(t) = frame.data {
            Ok(t)
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}
