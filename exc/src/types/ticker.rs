use derive_more::Display;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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
