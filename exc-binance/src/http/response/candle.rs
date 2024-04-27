use rust_decimal::Decimal;
use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

/// Candle in list form.
#[allow(dead_code)]
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

/// Options Candle.
#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionsCandle {
    /// Open.
    pub(crate) open: Decimal,
    /// High.
    pub(crate) high: Decimal,
    /// Low.
    pub(crate) low: Decimal,
    /// Close.
    pub(crate) close: Decimal,
    /// Volume.
    pub(crate) volume: Decimal,
    /// Amount.
    pub(crate) amount: Decimal,
    /// Interval.
    pub(crate) interval: String,
    /// Trade Count.
    pub(crate) trade_count: usize,
    /// Taker volume.
    pub(crate) taker_volume: Decimal,
    /// Taker amount.
    pub(crate) taker_amount: Decimal,
    /// Open time.
    pub(crate) open_time: i64,
    /// Close time.
    pub(crate) close_time: i64,
}

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

/// Candle Kind.
pub enum CandlesKind {
    /// Candles.
    Candles(Vec<Candle>),
    /// Options Candles.
    OptionsCandles(Vec<OptionsCandle>),
}

impl TryFrom<Data> for CandlesKind {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Candles(c) => Ok(Self::Candles(c)),
            Data::OptionsCandles(c) => Ok(Self::OptionsCandles(c)),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
