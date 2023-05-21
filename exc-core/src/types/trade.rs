use derive_more::Display;
use futures::stream::BoxStream;
use indicator::{Tick, TickValue, Tickable};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{ExchangeError, Request, Str};

/// Trade Stream.
pub type TradeStream = BoxStream<'static, Result<Trade, ExchangeError>>;

/// Trade.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display)]
#[display(fmt = "ts={ts} ({price}, {size}, {buy})")]
pub struct Trade {
    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// Price.
    pub price: Decimal,
    /// Size.
    pub size: Decimal,
    /// Is the taker of the buy side.
    pub buy: bool,
}

/// Subscribe trades.
#[derive(Debug, Clone)]
pub struct SubscribeTrades {
    /// Instrument.
    pub instrument: Str,
}

impl SubscribeTrades {
    /// Create a new [`SubscribeTrades`] request.
    pub fn new(inst: impl AsRef<str>) -> Self {
        Self {
            instrument: Str::new(inst),
        }
    }
}

impl Request for SubscribeTrades {
    type Response = TradeStream;
}

impl Tickable for Trade {
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
