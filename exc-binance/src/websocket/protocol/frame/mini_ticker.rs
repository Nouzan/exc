use rust_decimal::Decimal;
use serde::Deserialize;

use crate::websocket::error::WsError;

use super::{Name, Nameable, StreamFrame, StreamFrameKind};

/// 24hr rolling window mini-ticker statistics.
#[derive(Debug, Clone, Deserialize)]
pub struct MiniTicker {
    /// Event type.
    #[serde(rename = "e")]
    pub event: Option<String>,
    /// Event time.
    #[serde(rename = "E")]
    pub event_timestamp: Option<i64>,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Close price.
    #[serde(rename = "c")]
    pub close: Decimal,
    /// Open price.
    #[serde(rename = "o")]
    pub open: Decimal,
    /// High price.
    #[serde(rename = "h")]
    pub high: Decimal,
    /// Low price.
    #[serde(rename = "l")]
    pub low: Decimal,
    /// Total traded base asset volume.
    #[serde(rename = "v")]
    pub vol: Decimal,
    /// Total traded quote asset volume.
    #[serde(rename = "q")]
    pub vol_ccy: Decimal,
}

impl Nameable for MiniTicker {
    fn to_name(&self) -> Name {
        Name {
            inst: Some(self.symbol.to_lowercase()),
            channel: String::from("miniTicker"),
        }
    }
}

impl TryFrom<StreamFrame> for MiniTicker {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::MiniTicker(t) = frame.data {
            Ok(t)
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}
