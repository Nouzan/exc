use rust_decimal::Decimal;
use serde::Deserialize;

use crate::websocket::error::WsError;

use super::{Name, Nameable, StreamFrame, StreamFrameKind};

/// # Example
/// A [`Trade`] in JSON format:
/// ```json
/// {
///     "e": "trade",     // Event type
///     "E": 123456789,   // Event time
///     "s": "BTCUSDT",   // Symbol
///     "t": 12345,       // Trade ID
///     "p": "0.001",     // Price
///     "q": "100",       // Quantity
///     "b": 88,          // Buyer order ID
///     "a": 50,          // Seller order ID
///     "T": 1591677567872,   // Trade time (ms)
///     "S": "-1",        // "-1": sell; "1": buy
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    /// Event type.
    #[serde(rename = "e")]
    pub event: String,
    /// Event time.
    #[serde(rename = "E")]
    pub event_timestamp: i64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Trade ID.
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity.
    #[serde(rename = "q")]
    pub size: Decimal,
    /// Buyer order ID.
    #[serde(rename = "b")]
    pub buyer_order_id: i64,
    /// Seller order ID.
    #[serde(rename = "a")]
    pub seller_order_id: i64,
    /// Trade time (ms).
    #[serde(rename = "T")]
    pub trade_timestamp: i64,
    /// "-1": sell; "1": buy
    #[serde(rename = "S")]
    pub side: String,
}

impl Trade {
    /// Is buyer the market maker.
    pub fn is_taker_buy(&self) -> bool {
        self.side == "1"
    }

    /// Is taker sell.
    pub fn is_taker_sell(&self) -> bool {
        self.side == "-1"
    }
}

impl Nameable for Trade {
    fn to_name(&self) -> Name {
        Name {
            // FIXME: better way to determine the case (lower or upper).
            inst: Some(self.symbol.clone()),
            channel: self.event.clone(),
        }
    }
}

impl TryFrom<StreamFrame> for Trade {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::Trade(trade) = frame.data {
            Ok(trade)
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}
