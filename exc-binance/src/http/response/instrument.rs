#![allow(dead_code)]

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

/// Exchange info.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExchangeInfo {
    /// Usd-margin futures.
    UsdMarginFutures(UFExchangeInfo),
    /// Spot.
    Spot(SpotExchangeInfo),
}

/// Usd-margin futures exchange info.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UFExchangeInfo {
    pub(crate) exchange_filters: Vec<serde_json::Value>,
    pub(crate) rate_limits: Vec<RateLimit>,
    pub(crate) assets: Vec<serde_json::Value>,
    pub(crate) symbols: Vec<UFSymbol>,
    pub(crate) timezone: String,
}

/// Spot exchange info.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotExchangeInfo {
    pub(crate) exchange_filters: Vec<serde_json::Value>,
    pub(crate) rate_limits: Vec<RateLimit>,
    #[serde(default)]
    pub(crate) assets: Vec<serde_json::Value>,
    pub(crate) symbols: Vec<SpotSymbol>,
    pub(crate) timezone: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RateLimit {
    interval: String,
    interval_num: u64,
    limit: u64,
    rate_limit_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpotSymbol {
    pub(crate) symbol: String,
    pub(crate) status: String,
    pub(crate) base_asset: String,
    pub(crate) base_asset_precision: u32,
    pub(crate) quote_asset: String,
    pub(crate) quote_precision: u32,
    pub(crate) order_types: Vec<String>,
    pub(crate) iceberg_allowed: bool,
    pub(crate) oco_allowed: bool,
    pub(crate) quote_order_qty_market_allowed: bool,
    pub(crate) allow_trailing_stop: bool,
    pub(crate) cancel_replace_allowed: bool,
    pub(crate) is_spot_trading_allowed: bool,
    pub(crate) is_margin_trading_allowed: bool,
    pub(crate) filters: Vec<Filter>,
    pub(crate) permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UFSymbol {
    pub(crate) symbol: String,
    pub(crate) pair: String,
    pub(crate) contract_type: String,
    pub(crate) delivery_date: i64,
    pub(crate) onboard_date: i64,
    pub(crate) status: String,
    pub(crate) base_asset: String,
    pub(crate) quote_asset: String,
    pub(crate) margin_asset: String,
    pub(crate) price_precision: u32,
    pub(crate) quantity_precision: u32,
    pub(crate) base_asset_precision: u32,
    pub(crate) quote_precision: u32,
    pub(crate) underlying_type: String,
    pub(crate) settle_plan: i64,
    pub(crate) trigger_protect: Decimal,
    pub(crate) order_types: Vec<String>,
    pub(crate) time_in_force: Vec<String>,
    pub(crate) liquidation_fee: Decimal,
    pub(crate) market_take_bound: Decimal,
    pub(crate) filters: Vec<Filter>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Filter {
    /// Symbol.
    Symbol(SymbolFilter),
    /// Unknwon.
    Unknwon(serde_json::Value),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "filterType")]
pub(crate) enum SymbolFilter {
    #[serde(rename = "PRICE_FILTER")]
    PriceFilter {
        /// Max price.
        #[serde(rename = "maxPrice")]
        max_price: Decimal,
        /// Min price.
        #[serde(rename = "minPrice")]
        min_price: Decimal,
        /// Tick size.
        #[serde(rename = "tickSize")]
        tick_size: Decimal,
    },
    #[serde(rename = "LOT_SIZE")]
    LotSize {
        /// Max quantity.
        #[serde(rename = "maxQty")]
        max_qty: Decimal,
        /// Min quantity.
        #[serde(rename = "minQty")]
        min_qty: Decimal,
        /// step size.
        #[serde(rename = "stepSize")]
        step_size: Decimal,
    },
    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLotSize {
        /// Max quantity.
        #[serde(rename = "maxQty")]
        max_qty: Decimal,
        /// Min quantity.
        #[serde(rename = "minQty")]
        min_qty: Decimal,
        /// step size.
        #[serde(rename = "stepSize")]
        step_size: Decimal,
    },
    #[serde(rename = "MAX_NUM_ORDERS")]
    MaxNumOrders {
        /// Limit.
        limit: u64,
    },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    MaxNumAlgoOrders {
        /// Limit.
        limit: u64,
    },
    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional {
        #[serde(alias = "minNotional")]
        notional: Decimal,
    },
    #[serde(rename = "PERCENT_PRICE")]
    PercentPrice {
        #[serde(rename = "multiplierUp")]
        multiplier_up: Decimal,
        #[serde(rename = "multiplierDown")]
        multiplier_down: Decimal,
        #[serde(rename = "multiplierDecimal")]
        multiplier_decimal: Decimal,
    },
}

impl TryFrom<Data> for ExchangeInfo {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::ExchangeInfo(v) => Ok(v),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
