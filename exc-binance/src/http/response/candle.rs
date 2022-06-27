use rust_decimal::Decimal;
use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

/// Candle.
#[derive(Debug, Deserialize)]
pub struct Candle(
    /// Open time.
    pub(crate) i64,
    /// Open.
    pub(crate) Decimal,
    /// High.
    pub(crate) Decimal,
    /// Low.
    pub(crate) Decimal,
    /// Close.
    pub(crate) Decimal,
    /// Volume.
    pub(crate) Decimal,
    /// Close time.
    pub(crate) i64,
    /// Volume in quote.
    pub(crate) Decimal,
    /// The number of trades.
    pub(crate) usize,
    /// Taker volume.
    pub(crate) Decimal,
    /// Taker volume in quote.
    pub(crate) Decimal,
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
