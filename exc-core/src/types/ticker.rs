use crate::{ExchangeError, Request, Str};
use derive_more::Display;
use futures::stream::BoxStream;
use indicator::{Tick, TickValue, Tickable};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Ticker Stream.
pub type TickerStream = BoxStream<'static, Result<Ticker, ExchangeError>>;

/// Subscribe tickers.
#[derive(Debug, Clone)]
pub struct SubscribeTickers {
    /// Instrument.
    pub instrument: Str,
}

impl SubscribeTickers {
    /// Create a new [`SubscribeTickers`] request.
    pub fn new(inst: impl AsRef<str>) -> Self {
        Self {
            instrument: Str::new(inst),
        }
    }
}

impl Request for SubscribeTickers {
    type Response = TickerStream;
}

/// Ticker.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display)]
#[display(
    fmt = "ts={ts}, last=({last}, {size}), bid=({bid:?}, {bid_size:?}), ask=({ask:?}, {ask_size:?})"
)]
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
    /// The size of current best bid.
    #[serde(default)]
    pub bid_size: Option<Decimal>,
    /// Current best ask.
    #[serde(default)]
    pub ask: Option<Decimal>,
    /// The size of current best ask.
    #[serde(default)]
    pub ask_size: Option<Decimal>,
}

impl Tickable for Ticker {
    type Value = Self;

    fn tick(&self) -> Tick {
        Tick::new(self.ts)
    }

    fn value(&self) -> &Self::Value {
        self
    }

    fn into_tick_value(self) -> TickValue<Self::Value> {
        TickValue::new(self.ts, self)
    }
}

/// Statistic Stream.
pub type StatisticStream = BoxStream<'static, Result<Statistic, ExchangeError>>;

/// Subscribe tickers.
#[derive(Debug, Clone)]
pub struct SubscribeStatistics {
    /// Instrument.
    pub instrument: Str,
}

impl SubscribeStatistics {
    /// Create a new [`SubscribeStatistic`] request.
    pub fn new(inst: impl AsRef<str>) -> Self {
        Self {
            instrument: Str::new(inst),
        }
    }
}

impl Request for SubscribeStatistics {
    type Response = StatisticStream;
}

/// Statistic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display)]
#[display(fmt = "ts={ts}, close={close}, open={open:?}, high={high:?}, low={low:?}, vol={vol:?}")]
pub struct Statistic {
    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Last traded price.
    pub close: Decimal,
    /// Open price in the past 24 hours
    #[serde(default)]
    pub open: Decimal,
    /// Highest price in the past 24 hours
    #[serde(default)]
    pub high: Decimal,
    /// Lowest price in the past 24 hours
    #[serde(default)]
    pub low: Decimal,
    /// 24h trading volume.
    #[serde(default)]
    pub vol: Decimal,
}
