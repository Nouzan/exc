use rust_decimal::Decimal;
use serde::Deserialize;

use super::{Name, Nameable};

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
    #[serde(rename = "e")]
    event: String,
    #[serde(rename = "E")]
    event_timestamp: i64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "a")]
    aggregate_id: usize,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "q")]
    size: Decimal,
    #[serde(rename = "f")]
    first_id: usize,
    #[serde(rename = "l")]
    last_id: usize,
    #[serde(rename = "T")]
    trade_timestamp: i64,
    #[serde(rename = "m")]
    buy_maker: bool,
}

impl Nameable for AggTrade {
    fn to_name(&self) -> Name {
        Name {
            inst: self.symbol.to_lowercase(),
            channel: self.event.clone(),
        }
    }
}
