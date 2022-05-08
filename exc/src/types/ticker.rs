use super::Request;
use crate::ExchangeError;
use derive_more::Display;
use futures::stream::BoxStream;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Ticker Stream.
pub type TickerStream = BoxStream<'static, Result<Ticker, ExchangeError>>;

/// Subscribe tickers.
#[derive(Debug, Clone)]
pub struct SubscribeTickers {
    /// Instrument.
    pub instrument: String,
}

impl SubscribeTickers {
    /// Create a new [`SubscribeTickers`]
    pub fn new(inst: &str) -> Self {
        Self {
            instrument: inst.to_string(),
        }
    }
}

impl Request for SubscribeTickers {
    type Response = TickerStream;
}

/// Ticker.
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[display(fmt = "ts={ts} ({last}, {size}) bid={bid:?} ask={ask:?}")]
pub struct Ticker {
    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Last traded price.
    pub last: Decimal,
    /// Last traded size.
    #[serde(default)]
    pub size: Decimal,
    /// Last traded side.
    #[serde(default)]
    pub buy: Option<bool>,
    /// Current best bid.
    #[serde(default)]
    pub bid: Option<Decimal>,
    /// Current bestt ask.
    #[serde(default)]
    pub ask: Option<Decimal>,
}
