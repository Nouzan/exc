use std::fmt;

use crate::Str;
use futures::stream::BoxStream;
use indicator::{Tick, TickValue, Tickable};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use exc_service::{ExchangeError, Request};

/// Best bid and ask Stream.
pub type BidAskStream = BoxStream<'static, Result<BidAsk, ExchangeError>>;

/// Current best bid and best ask.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BidAsk {
    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Current best bid (price, size).
    pub bid: Option<(Decimal, Decimal)>,
    /// Current best ask (price, size).
    pub ask: Option<(Decimal, Decimal)>,
}

impl fmt::Display for BidAsk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.bid, self.ask) {
            (Some((bid, bid_size)), Some((ask, ask_size))) => write!(
                f,
                "ts={}, bid=({bid}, {bid_size}), ask=({ask}, {ask_size})",
                self.ts
            ),
            (Some((bid, bid_size)), None) => {
                write!(f, "ts={}, bid=({bid}, {bid_size}), ask=null", self.ts)
            }
            (None, Some((ask, ask_size))) => {
                write!(f, "ts={}, bid=null, ask=({ask}, {ask_size})", self.ts)
            }
            _ => write!(f, "ts={}, bid=null, ask=null", self.ts),
        }
    }
}

impl Tickable for BidAsk {
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

/// Subscribe current best bid and ask.
#[derive(Debug, Clone)]
pub struct SubscribeBidAsk {
    /// Instrument.
    pub instrument: Str,
}

impl SubscribeBidAsk {
    /// Create a new [`SubscribeBidAsk`] request.
    pub fn new(inst: impl AsRef<str>) -> Self {
        Self {
            instrument: Str::new(inst),
        }
    }
}

impl Request for SubscribeBidAsk {
    type Response = BidAskStream;
}
