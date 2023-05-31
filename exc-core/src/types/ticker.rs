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
    /// Open price in the past 24 hours
    #[serde(default)]
    pub open_24h: Option<Decimal>,
    /// Highest price in the past 24 hours
    #[serde(default)]
    pub high_24h: Option<Decimal>,
    /// Lowest price in the past 24 hours
    #[serde(default)]
    pub low_24h: Option<Decimal>,
    /// 24h trading volume, with a unit of currency.
    #[serde(default)]
    pub vol_ccy_24h: Option<Decimal>,
    /// 24h trading volume, with a unit of contract.
    #[serde(default)]
    pub vol_24h: Option<Decimal>,
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

/// Mini Ticker Stream.
pub type MiniTickerStream = BoxStream<'static, Result<MiniTicker, ExchangeError>>;

/// Subscribe tickers.
#[derive(Debug, Clone)]
pub struct SubscribeMiniTickers {
    /// Instrument.
    pub instrument: Str,
}

impl SubscribeMiniTickers {
    /// Create a new [`SubscribeMiniTicker`] request.
    pub fn new(inst: impl AsRef<str>) -> Self {
        Self {
            instrument: Str::new(inst),
        }
    }
}

impl Request for SubscribeMiniTickers {
    type Response = MiniTickerStream;
}

/// Mini Ticker.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display)]
#[display(
    fmt = "ts={ts}, last={last}, open={open_24h:?}, high={high_24h:?}, low={low_24h:?}, vol_ccy={vol_ccy_24h:?}, vol={vol_24h:?}"
)]
pub struct MiniTicker {
    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Last traded price.
    pub last: Decimal,
    /// Open price in the past 24 hours
    #[serde(default)]
    pub open_24h: Decimal,
    /// Highest price in the past 24 hours
    #[serde(default)]
    pub high_24h: Decimal,
    /// Lowest price in the past 24 hours
    #[serde(default)]
    pub low_24h: Decimal,
    /// 24h trading volume, with a unit of currency.
    #[serde(default)]
    pub vol_ccy_24h: Decimal,
    /// 24h trading volume, with a unit of contract.
    #[serde(default)]
    pub vol_24h: Decimal,
}
