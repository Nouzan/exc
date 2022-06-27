use derive_more::Display;
use futures::stream::BoxStream;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{ExchangeError, Request};

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
    pub instrument: String,
}

impl Request for SubscribeTrades {
    type Response = TradeStream;
}
