use rust_decimal::Decimal;
use serde::Deserialize;

use crate::websocket::error::WsError;

use super::{Name, Nameable, StreamFrame};

/// # Example
/// A [`AggTrade`] in JSON format:
/// ```json
/// {
///     "e": "aggTrade",  // Event type
///     "E": 123456789,   // Event time
///     "s": "BTCUSDT",   // Symbol
///     "a": 5933014,     // Aggregate trade ID
///     "p": "0.001",     // Price
///     "q": "100",       // Quantity
///     "f": 100,         // First trade ID
///     "l": 105,         // Last trade ID
///     "T": 123456785,   // Trade time
///     "m": true,        // Is the buyer the market maker?
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AggTrade {
    /// Event type.
    #[serde(rename = "e")]
    pub event: String,
    /// Event time.
    #[serde(rename = "E")]
    pub event_timestamp: i64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Aggregate trade ID.
    #[serde(rename = "a")]
    pub aggregate_id: usize,
    /// Price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity.
    #[serde(rename = "q")]
    pub size: Decimal,
    /// First trade ID.
    #[serde(rename = "f")]
    pub first_id: usize,
    /// Last trade ID.
    #[serde(rename = "l")]
    pub last_id: usize,
    /// Trade time.
    #[serde(rename = "T")]
    pub trade_timestamp: i64,
    /// Is the buyer the market maker.
    #[serde(rename = "m")]
    pub buy_maker: bool,
}

impl Nameable for AggTrade {
    fn to_name(&self) -> Name {
        Name {
            inst: self.symbol.to_lowercase(),
            channel: self.event.clone(),
        }
    }
}

impl TryFrom<StreamFrame> for AggTrade {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrame::AggTrade(trade) = frame {
            Ok(trade)
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}
