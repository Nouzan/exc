use rust_decimal::Decimal;
use serde::Deserialize;

use crate::websocket::error::WsError;

use super::{StreamFrame, StreamFrameKind};

/// Depth.
#[derive(Debug, Clone, Deserialize)]
pub struct Depth {
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
    /// Update ID.
    #[serde(rename = "u")]
    pub id: usize,
    /// Bids.
    #[serde(rename = "b")]
    pub bids: Vec<(Decimal, Decimal)>,
    /// Asks.
    #[serde(rename = "a")]
    pub asks: Vec<(Decimal, Decimal)>,
}

impl TryFrom<StreamFrame> for Depth {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::Depth(t) = frame.data {
            Ok(t)
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}
