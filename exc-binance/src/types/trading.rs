use exc_core::{types, ExchangeError};
use serde::{Deserialize, Serialize};

/// Order side.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    /// Buy.
    Buy,
    /// Sell.
    Sell,
}

/// Position side.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionSide {
    /// Long.
    Long,
    /// Short.
    Short,
    /// Both.
    Both,
}

/// Time-in-force.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    /// Good-Till-Cancel.
    Gtc,
    /// Immdiate-Or-Cancel.
    Ioc,
    /// Fill-Or-Kill.
    Fok,
    /// Good-Till-Cancel (Post-Only)
    Gtx,
}

/// Status.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    /// New.
    #[serde(alias = "ACCEPTED")]
    New,
    /// Parttially filled.
    PartiallyFilled,
    /// Filled.
    Filled,
    /// Cancelled.
    #[serde(alias = "REJECTED", alias = "CANCELLED")]
    Canceled,
    /// Expired.
    Expired,
    /// New insurance.
    NewInsurance,
    /// New ADL.
    NewAdl,
}

impl TryFrom<Status> for types::OrderStatus {
    type Error = ExchangeError;

    fn try_from(status: Status) -> Result<Self, Self::Error> {
        let status = match status {
            Status::New | Status::PartiallyFilled => types::OrderStatus::Pending,
            Status::Canceled | Status::Expired | Status::Filled => types::OrderStatus::Finished,
            Status::NewAdl | Status::NewInsurance => types::OrderStatus::Pending,
        };
        Ok(status)
    }
}

/// Order type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    /// Market.
    Market,
    /// Limit.
    Limit,
    /// Stop.
    Stop,
    /// Take-Profit.
    TakeProfit,
    /// Stop-Market.
    StopMarket,
    /// Take-Profit-Market.
    TakeProfitMarket,
    /// Trailing-Stop-Market.
    TrailingStopMarket,
    /// Limit Maker.
    LimitMaker,
}
