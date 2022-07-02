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
#[serde(rename_all = "camelCase")]
pub struct Order {
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

impl TryFrom<Data> for Order {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Order(order) => Ok(order),
            Data::Error(msg) => match msg.code {
                -2013 => Err(RestError::Exchange(ExchangeError::OrderNotFound)),
                err => Err(RestError::Exchange(ExchangeError::Api(anyhow::anyhow!(
                    "{err:?}"
                )))),
            },
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
