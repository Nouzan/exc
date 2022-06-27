use rust_decimal::Decimal;
use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

/// Candle.
#[derive(Debug, Deserialize)]
pub struct Candle(
    /// Open time.
    i64,
    /// Open.
    Decimal,
    /// High.
    Decimal,
    /// Low.
    Decimal,
    /// Close.
    Decimal,
    /// Volume.
    Decimal,
    /// Close time.
    i64,
    /// Volume in quote.
    Decimal,
    /// The number of trades.
    usize,
    /// Taker volume.
    Decimal,
    /// Taker volume in quote.
    Decimal,
    /// Ignore.
    serde_json::Value,
);

impl TryFrom<Data> for Vec<Candle> {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Candles(c) => Ok(c),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
