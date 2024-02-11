use exc_core::{Asset, ExchangeError, Str};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_with::serde_as;

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
    /// Options.
    EuropeanOptions(OptionsOrder),
    /// Spot.
    Spot(SpotOrder),
}

impl Order {
    /// Get order id.
    pub fn id(&self) -> i64 {
        match self {
            Self::UsdMarginFutures(order) => order.order_id,
            Self::Spot(order) => order.ack.order_id,
            Self::EuropeanOptions(order) => order.order_id,
        }
    }

    /// Get symbol.
    pub fn symbol(&self) -> &str {
        match self {
            Self::UsdMarginFutures(order) => order.symbol.as_str(),
            Self::Spot(order) => order.ack.symbol.as_str(),
            Self::EuropeanOptions(order) => order.symbol.as_str(),
        }
    }

    /// Get client order id.
    pub fn client_id(&self) -> &str {
        tracing::debug!("get client id; {self:?}");
        match self {
            Self::UsdMarginFutures(order) => order.client_order_id.as_str(),
            Self::Spot(order) => order.ack.client_order_id(),
            Self::EuropeanOptions(order) => order
                .client_order_id
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        }
    }

    /// Get updated time.
    pub fn updated(&self) -> Option<i64> {
        match self {
            Self::UsdMarginFutures(order) => Some(order.update_time),
            Self::Spot(order) => order.ack.transact_time,
            Self::EuropeanOptions(order) => order.state.as_ref().map(|s| s.update_time),
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
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotAck {
    /// Symbol.
    pub symbol: String,
    /// Order id.
    #[serde_as(as = "serde_with::PickFirst<(_, serde_with::DisplayFromStr)>")]
    pub order_id: i64,
    /// Orignal client order id.
    orig_client_order_id: Option<String>,
    /// Client id.
    client_order_id: String,
    /// Update timestamp.
    #[serde(alias = "updateTime")]
    pub transact_time: Option<i64>,
}

impl SpotAck {
    /// Client order id.
    pub fn client_order_id(&self) -> &str {
        match &self.orig_client_order_id {
            Some(id) => id.as_str(),
            None => self.client_order_id.as_str(),
        }
    }
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
    pub commission_asset: Asset,
    /// Trade id.
    pub trade_id: i64,
}

/// Options Order.
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionsOrder {
    /// Order id.
    pub(crate) order_id: i64,
    /// Client id.
    pub(crate) client_order_id: Option<Str>,
    /// Symbol.
    pub(crate) symbol: Str,
    /// Price.
    pub(crate) price: Decimal,
    /// Size.
    pub(crate) quantity: Decimal,
    /// Side.
    pub(crate) side: OrderSide,
    /// Order type.
    #[serde(rename = "type")]
    pub(crate) order_type: OrderType,
    /// Reduce only.
    #[allow(unused)]
    pub(crate) reduce_only: bool,
    /// Post only.
    pub(crate) post_only: bool,
    /// Mmp.
    #[allow(unused)]
    pub(crate) mmp: bool,
    /// State.
    #[serde(flatten, default)]
    pub(crate) state: Option<OptionsOrderState>,
}

/// Options Order State.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionsOrderState {
    /// Create time.
    #[allow(unused)]
    pub(crate) create_time: i64,
    /// Update time.
    pub(crate) update_time: i64,
    /// Filled size.
    pub(crate) executed_qty: Decimal,
    /// Average price.
    pub(crate) avg_price: Decimal,
    /// Quote asset.
    pub(crate) quote_asset: Asset,
    /// Fee.
    pub(crate) fee: Decimal,
    /// Time-In-Force.
    pub(crate) time_in_force: TimeInForce,
    /// Status.
    pub(crate) status: Status,
}
