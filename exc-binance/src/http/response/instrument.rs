#![allow(dead_code)]

use exc_core::{symbol::ExcSymbol, Asset, Str};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

const TRADING: &str = "TRADING";

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
    pub(crate) base_asset: Asset,
    pub(crate) base_asset_precision: u32,
    pub(crate) quote_asset: Asset,
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

impl SpotSymbol {
    pub(crate) fn to_exc_symbol(&self) -> Result<ExcSymbol, RestError> {
        Ok(ExcSymbol::spot(&self.base_asset, &self.quote_asset))
    }

    pub(crate) fn is_live(&self) -> bool {
        self.status == TRADING
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum KnownContractType {
    Perpetual,
    CurrentQuarter,
    NextQuarter,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ContractType {
    Known(KnownContractType),
    Unknwon(String),
}

impl ContractType {
    const U_EMPTY: Str = Str::new_inline("U-EMPTY");
    const U_PERP: Str = Str::new_inline("U-PERP");
    const U_CURRENT_QUARTER: Str = Str::new_inline("U-QUARTER");
    const U_NEXT_QUARTER: Str = Str::new_inline("U-NEXT-QUARTER");

    fn to_prefix(&self) -> Str {
        match self {
            Self::Known(c) => match c {
                KnownContractType::Perpetual => Self::U_PERP,
                KnownContractType::CurrentQuarter => Self::U_CURRENT_QUARTER,
                KnownContractType::NextQuarter => Self::U_NEXT_QUARTER,
            },
            Self::Unknwon(s) => {
                if s.is_empty() {
                    Self::U_EMPTY
                } else {
                    Str::new(format!("U-{s}"))
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UFSymbol {
    pub(crate) symbol: String,
    pub(crate) pair: String,
    pub(crate) contract_type: ContractType,
    pub(crate) delivery_date: i64,
    pub(crate) onboard_date: i64,
    pub(crate) status: String,
    pub(crate) base_asset: Asset,
    pub(crate) quote_asset: Asset,
    pub(crate) margin_asset: Asset,
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

impl UFSymbol {
    pub(crate) fn is_live(&self) -> bool {
        self.status == TRADING
    }

    pub(crate) fn to_exc_symbol(&self) -> Result<ExcSymbol, RestError> {
        match &self.contract_type {
            ContractType::Known(ty) => match ty {
                KnownContractType::Perpetual => {
                    Ok(ExcSymbol::perpetual(&self.base_asset, &self.quote_asset))
                }
                KnownContractType::NextQuarter | KnownContractType::CurrentQuarter => {
                    let date = self
                        .symbol
                        .split('_')
                        .last()
                        .ok_or(RestError::MissingDateForFutures)?;
                    ExcSymbol::futures_with_str(&self.base_asset, &self.quote_asset, date)
                        .ok_or(RestError::FailedToBuildExcSymbol)
                }
            },
            ContractType::Unknwon(ty) => Err(RestError::UnknownContractType(ty.clone())),
        }
    }
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
    #[serde(rename = "NOTIONAL")]
    Notional {
        #[serde(alias = "minNotional")]
        min_notional: Decimal,
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
