use exc_core::ExchangeError;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    http::error::RestError,
    types::trading::{OrderSide, OrderType, PositionSide, Status, TimeInForce},
};

use super::Data;

/// Order.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Order {
    /// Usd-Margin Futures.
    UsdMarginFutures(UsdMarginFuturesOrder),
    /// Spot.
    Spot(SpotOrder),
}

impl Order {
    /// Get order id.
    pub fn id(&self) -> i64 {
        match self {
            Self::UsdMarginFutures(order) => order.order_id,
            Self::Spot(order) => order.ack.order_id,
        }
    }

    /// Get symbol.
    pub fn symbol(&self) -> &str {
        match self {
            Self::UsdMarginFutures(order) => order.symbol.as_str(),
            Self::Spot(order) => order.ack.symbol.as_str(),
        }
    }

    /// Get client order id.
    pub fn client_id(&self) -> &str {
        match self {
            Self::UsdMarginFutures(order) => order.client_order_id.as_str(),
            Self::Spot(order) => order.ack.client_order_id.as_str(),
        }
    }

    /// Get updated time.
    pub fn updated(&self) -> Option<i64> {
        match self {
            Self::UsdMarginFutures(order) => Some(order.update_time),
            Self::Spot(order) => order.ack.transact_time,
        }
    }
}

impl TryFrom<Data> for Order {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Order(order) => Ok(order),
            Data::Error(msg) => match msg.code {
                -2013 => Err(RestError::Exchange(ExchangeError::OrderNotFound)),
                _ => Err(RestError::Exchange(ExchangeError::Api(anyhow::anyhow!(
                    "{msg:?}"
                )))),
            },
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}

/// Usd-Margin Futures Order.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsdMarginFuturesOrder {
    /// Client id.
    pub client_order_id: String,
    /// FIXME: what is this?
    pub cum_qty: Option<Decimal>,
    /// FIXME: what is this?
    pub cum_quote: Option<Decimal>,
    /// Filled size.
    pub executed_qty: Decimal,
    /// Order id.
    pub order_id: i64,
    /// Cost.
    pub avg_price: Decimal,
    /// Size.
    pub orig_qty: Decimal,
    /// Price.
    pub price: Decimal,
    /// Reduce only.
    pub reduce_only: bool,
    /// Order side.
    pub side: OrderSide,
    /// Position side.
    pub position_side: PositionSide,
    /// Status.
    pub status: Status,
    /// Stop price.
    pub stop_price: Decimal,
    /// Is close position.
    pub close_position: bool,
    /// Symbol.
    pub symbol: String,
    /// Time-In-Force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Active price.
    pub activate_price: Option<Decimal>,
    /// Price rate.
    pub price_rate: Option<Decimal>,
    /// Update timestamp.
    pub update_time: i64,
    /// Working type.
    pub working_type: String,
    /// Price protect.
    pub price_protect: bool,
}

/// Spot Ack.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotAck {
    /// Symbol.
    pub symbol: String,
    /// Order id.
    pub order_id: i64,
    /// Client id.
    pub client_order_id: String,
    /// Update timestamp.
    #[serde(alias = "updateTime")]
    pub transact_time: Option<i64>,
}

/// Spot Result.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotResult {
    /// Price.
    pub price: Decimal,
    /// Size.
    pub orig_qty: Decimal,
    /// Filled size.
    pub executed_qty: Decimal,
    /// Filled quote size.
    pub cummulative_quote_qty: Decimal,
    /// Status.
    pub status: Status,
    /// Time-In-Force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
}

/// Spot Order.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotOrder {
    /// Ack.
    #[serde(flatten)]
    pub ack: SpotAck,

    /// Result.
    #[serde(flatten)]
    pub result: Option<SpotResult>,

    /// Fills.
    #[serde(default)]
    pub fills: Vec<SpotFill>,
}

/// Spot fill.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotFill {
    /// Price.
    pub price: Decimal,
    /// Size.
    pub qty: Decimal,
    /// Fee.
    pub commission: Decimal,
    /// Fee asset.
    pub commission_asset: String,
    /// Trade id.
    pub trade_id: i64,
}
