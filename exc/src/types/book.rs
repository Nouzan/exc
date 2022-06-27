use std::fmt;

use futures::stream::BoxStream;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{ExchangeError, Request};

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

/// Subscribe current best bid and ask.
#[derive(Debug, Clone)]
pub struct SubscribeBidAsk {
    /// Instrument.
    pub instrument: String,
}

impl Request for SubscribeBidAsk {
    type Response = BidAsk;
}
