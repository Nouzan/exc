use std::{collections::HashMap, ops::Neg};

use either::Either;
use exc_core::{types, Asset, ExchangeError, Str};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    types::trading::{OrderSide, PositionSide, Status, TimeInForce},
    websocket::error::WsError,
};

use super::{Name, Nameable, StreamFrame, StreamFrameKind};

/// Account events.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "e", rename_all = "camelCase")]
pub enum AccountEvent {
    /// Listen key expired.
    ListenKeyExpired {
        /// Event timestamp.
        #[serde(rename = "E")]
        ts: i64,
    },
    /// Order trade update.
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    OrderTradeUpdate {
        /// Event timestamp.
        #[serde(rename = "E")]
        event_ts: i64,
        /// Trade timestamp.
        #[serde(rename = "T")]
        trade_ts: i64,
        /// Order.
        #[serde(rename = "o")]
        order: OrderUpdate,
    },
    /// Order update (for spot).
    #[serde(rename = "executionReport")]
    ExecutionReport(ExecutionReport),
}

/// Order Update Frame.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum OrderUpdateFrame {
    /// Options.
    Options(OptionsOrder),
    /// USD-M Futures.
    UsdMarginFutures(OrderUpdate),
    /// Execution report.
    Spot(ExecutionReport),
}

/// Order type.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    /// Market.
    Market,
    /// Limit.
    Limit,
    /// Stop.
    Stop,
    /// Take profit.
    TakeProfit,
    /// Liquidation.
    Liquidation,
    /// Limit maker.
    LimitMaker,
}

/// Update kind.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UpdateKind {
    /// New.
    New,
    /// Cancelled.
    Canceled,
    /// Calculated.
    Calculated,
    /// Expired.
    Expired,
    /// Trade.
    Trade,
}

/// Order update.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderUpdate {
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: Str,
    /// Client id.
    #[serde(rename = "c")]
    pub client_id: Str,
    /// Order side.
    #[serde(rename = "S")]
    pub side: OrderSide,
    /// Order type.
    #[serde(rename = "o")]
    pub order_type: OrderType,
    /// Time-in-force.
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    /// Size.
    #[serde(rename = "q")]
    pub size: Decimal,
    /// Price. (FIXME: should this to be optional?)
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Cost.
    #[serde(rename = "ap")]
    pub cost: Decimal,
    /// Trigger price.
    #[serde(rename = "sp")]
    pub trigger_price: Decimal,
    /// Update kind.
    #[serde(rename = "x")]
    pub kind: UpdateKind,
    /// Status.
    #[serde(rename = "X")]
    pub status: Status,
    /// Order id.
    #[serde(rename = "i")]
    pub order_id: i64,
    /// Last trade size.
    #[serde(rename = "l")]
    pub last_trade_size: Decimal,
    /// Filled size.
    #[serde(rename = "z")]
    pub filled_size: Decimal,
    /// Last trade price.
    #[serde(rename = "L")]
    pub last_trade_price: Decimal,
    /// Fee asset.
    #[serde(rename = "N")]
    pub fee_asset: Option<Asset>,
    /// Fee.
    #[serde(rename = "n", default)]
    pub fee: Decimal,
    /// Trade timestamp.
    #[serde(rename = "T")]
    pub trade_ts: i64,
    /// Trade id.
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Bid equity.
    #[serde(rename = "b")]
    pub bid_equity: Decimal,
    /// Ask equity.
    #[serde(rename = "a")]
    pub ask_equity: Decimal,
    /// Maker.
    #[serde(rename = "m")]
    pub marker: bool,
    /// Reduce-Only.
    #[serde(rename = "R")]
    pub reduce_only: bool,
    /// Trigger type.
    #[serde(rename = "wt")]
    pub trigger_type: String,
    /// Original order type.
    #[serde(rename = "ot")]
    pub original_order_type: OrderType,
    /// Position side.
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
    /// Is triggered close.
    #[serde(rename = "cp")]
    pub is_triggered_close: Option<bool>,
    /// Active price.
    #[serde(rename = "AP")]
    pub active_price: Option<Decimal>,
    /// Cr.
    #[serde(rename = "cr")]
    pub cr: Option<Decimal>,
    /// Profit and loss.
    #[serde(rename = "rp")]
    pub pnl: Decimal,
}

/// Order update for spot.
#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionReport {
    /// Event timestamp.
    #[serde(rename = "E")]
    pub event_ts: i64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: Str,
    /// Client id.
    #[serde(rename = "c")]
    pub client_id: Str,
    /// Orignal Client id.
    #[serde(rename = "C")]
    pub orignal_client_id: Str,
    /// Order side.
    #[serde(rename = "S")]
    pub side: OrderSide,
    /// Order type.
    #[serde(rename = "o")]
    pub order_type: OrderType,
    /// Time-in-force.
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    /// Quote size.
    #[serde(rename = "Q")]
    pub quote_size: Decimal,
    /// Size.
    #[serde(rename = "q")]
    pub size: Decimal,
    /// Price. (FIXME: should this to be optional?)
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Update kind.
    #[serde(rename = "x")]
    pub kind: UpdateKind,
    /// Status.
    #[serde(rename = "X")]
    pub status: Status,
    /// Order id.
    #[serde(rename = "i")]
    pub order_id: i64,
    /// Last trade size.
    #[serde(rename = "l")]
    pub last_trade_size: Decimal,
    /// Last trade money.
    #[serde(rename = "Y")]
    pub last_trade_quote_size: Decimal,
    /// Filled size.
    #[serde(rename = "z")]
    pub filled_size: Decimal,
    /// Filled money.
    #[serde(rename = "Z")]
    pub filled_quote_size: Decimal,
    /// Last trade price.
    #[serde(rename = "L")]
    pub last_trade_price: Decimal,
    /// Fee asset.
    #[serde(rename = "N")]
    pub fee_asset: Option<Asset>,
    /// Fee.
    #[serde(rename = "n", default)]
    pub fee: Decimal,
    /// Trade timestamp.
    #[serde(rename = "T")]
    pub trade_ts: i64,
    /// Trade id.
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Maker.
    #[serde(rename = "m")]
    pub marker: bool,
    /// Created timestamp.
    #[serde(rename = "O")]
    pub create_ts: i64,
}

impl ExecutionReport {
    /// Get client id (original).
    pub fn client_id(&self) -> &str {
        if self.orignal_client_id.is_empty() {
            self.client_id.as_str()
        } else {
            self.orignal_client_id.as_str()
        }
    }
}

/// Order update for options.
#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct OptionsOrderUpdate {
    /// Event timestamp.
    #[serde(rename = "E")]
    pub(crate) event_ts: i64,
    #[serde(rename = "o")]
    pub(crate) order: Vec<OptionsOrder>,
}

/// Options order.
#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct OptionsOrder {
    /// Created timestamp.
    #[serde(rename = "T")]
    pub(crate) create_ts: i64,
    /// Updated timestamp.
    #[serde(rename = "t")]
    pub(crate) update_ts: i64,
    /// Symbol.
    #[serde(rename = "s")]
    pub(crate) symbol: Str,
    /// Client id.
    #[serde(rename = "c")]
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub(crate) client_id: Option<Str>,
    /// Order id.
    #[serde(rename = "oid")]
    pub(crate) order_id: Str,
    /// Price.
    #[serde(rename = "p")]
    pub(crate) price: Decimal,
    /// Size.
    #[serde(rename = "q")]
    pub(crate) size: Decimal,
    /// Reduce-only.
    #[serde(rename = "r")]
    pub(crate) reduce_only: bool,
    /// Post-only.
    #[serde(rename = "po")]
    pub(crate) post_only: bool,
    /// Status.
    #[serde(rename = "S")]
    pub(crate) status: Status,
    /// Filled size.
    #[serde(rename = "e")]
    pub(crate) filled_size: Decimal,
    /// Filled amount.
    #[serde(rename = "ec")]
    pub(crate) filled_amount: Decimal,
    /// Fee.
    #[serde(rename = "f")]
    pub(crate) fee: Decimal,
    /// Time-in-force.
    #[serde(rename = "tif")]
    pub(crate) time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "oty")]
    pub(crate) order_type: OrderType,
    /// Trades.
    #[serde(rename = "fi")]
    #[serde(default)]
    pub(crate) trades: Vec<OptionsTrade>,
}

/// Options trade.
#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct OptionsTrade {
    /// Trade ts.
    #[serde(rename = "T")]
    pub(crate) trade_ts: i64,
    /// Trade id.
    #[serde(rename = "t")]
    pub(crate) trade_id: Str,
    /// Size.
    #[serde(rename = "q")]
    pub(crate) size: Decimal,
    /// Price.
    #[serde(rename = "p")]
    pub(crate) price: Decimal,
    /// Fee.
    #[serde(rename = "f")]
    pub(crate) fee: Decimal,
    /// Maker or taker.
    #[serde(rename = "m")]
    pub(crate) maker: Str,
}

impl Nameable for OptionsOrder {
    fn to_name(&self) -> Name {
        Name::order_trade_update(&self.symbol)
    }
}

impl Nameable for AccountEvent {
    fn to_name(&self) -> Name {
        match self {
            Self::ListenKeyExpired { .. } => Name::listen_key_expired(),
            Self::OrderTradeUpdate { order, .. } => {
                Name::order_trade_update(&order.symbol.to_lowercase())
            }
            Self::ExecutionReport(r) => Name::order_trade_update(&r.symbol.to_lowercase()),
        }
    }
}

impl TryFrom<StreamFrame> for AccountEvent {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::AccountEvent(e) = frame.data {
            Ok(e)
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}

impl TryFrom<StreamFrame> for Either<OrderUpdate, ExecutionReport> {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::AccountEvent(e) = frame.data {
            match e {
                AccountEvent::OrderTradeUpdate { order, .. } => Ok(Either::Left(order)),
                AccountEvent::ExecutionReport(r) => Ok(Either::Right(r)),
                e => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{e:?}"))),
            }
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}

impl TryFrom<StreamFrame> for OrderUpdateFrame {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        match frame.data {
            StreamFrameKind::AccountEvent(e) => match e {
                AccountEvent::OrderTradeUpdate { order, .. } => Ok(Self::UsdMarginFutures(order)),
                AccountEvent::ExecutionReport(r) => Ok(Self::Spot(r)),
                e => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{e:?}"))),
            },
            StreamFrameKind::OptionsOrder(order) => Ok(Self::Options(order)),
            e => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{e:?}"))),
        }
    }
}

impl TryFrom<OrderUpdateFrame> for types::OrderUpdate {
    type Error = ExchangeError;

    fn try_from(value: OrderUpdateFrame) -> Result<Self, Self::Error> {
        match value {
            OrderUpdateFrame::UsdMarginFutures(update) => {
                let kind = match update.order_type {
                    OrderType::Limit => match update.time_in_force {
                        TimeInForce::Gtc => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::GoodTilCancelled,
                        ),
                        TimeInForce::Fok => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::FillOrKill,
                        ),
                        TimeInForce::Ioc => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::ImmediateOrCancel,
                        ),
                        TimeInForce::Gtx => types::OrderKind::PostOnly(update.price.normalize()),
                    },
                    OrderType::Market => types::OrderKind::Market,
                    other => {
                        return Err(ExchangeError::Other(anyhow!(
                            "unsupported order type: {other:?}"
                        )));
                    }
                };
                let mut filled = update.filled_size.abs().normalize();
                let mut size = update.size.abs().normalize();
                match update.side {
                    OrderSide::Buy => {
                        filled.set_sign_positive(true);
                        size.set_sign_positive(true)
                    }
                    OrderSide::Sell => {
                        filled.set_sign_positive(false);
                        size.set_sign_positive(false)
                    }
                }
                let status = update.status.try_into()?;
                let trade_size = update.last_trade_size.abs().normalize();
                let trade = if !trade_size.is_zero() {
                    let mut trade = types::OrderTrade {
                        price: update.last_trade_price.normalize(),
                        size: if matches!(update.side, OrderSide::Buy) {
                            trade_size
                        } else {
                            -trade_size
                        },
                        fee: Decimal::ZERO,
                        fee_asset: None,
                    };
                    if let Some(asset) = update.fee_asset {
                        trade.fee = -update.fee.normalize();
                        trade.fee_asset = Some(asset);
                    }
                    Some(trade)
                } else {
                    None
                };
                Ok(types::OrderUpdate {
                    ts: crate::types::adaptations::from_timestamp(update.trade_ts)?,
                    order: types::Order {
                        id: types::OrderId::from(update.client_id),
                        target: types::Place { size, kind },
                        state: types::OrderState {
                            filled,
                            cost: if filled.is_zero() {
                                Decimal::ONE
                            } else {
                                update.cost
                            },
                            status,
                            fees: HashMap::default(),
                        },
                        trade,
                    },
                })
            }
            OrderUpdateFrame::Spot(update) => {
                let client_id = update.client_id().to_string();
                let kind = match update.order_type {
                    OrderType::Limit => match update.time_in_force {
                        TimeInForce::Gtc => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::GoodTilCancelled,
                        ),
                        TimeInForce::Fok => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::FillOrKill,
                        ),
                        TimeInForce::Ioc => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::ImmediateOrCancel,
                        ),
                        TimeInForce::Gtx => types::OrderKind::PostOnly(update.price.normalize()),
                    },
                    OrderType::Market => types::OrderKind::Market,
                    OrderType::LimitMaker => types::OrderKind::PostOnly(update.price.normalize()),
                    other => {
                        return Err(ExchangeError::Other(anyhow!(
                            "unsupported order type: {other:?}"
                        )));
                    }
                };
                let mut filled = update.filled_size.abs().normalize();
                let mut size = update.size.abs().normalize();
                match update.side {
                    OrderSide::Buy => {
                        filled.set_sign_positive(true);
                        size.set_sign_positive(true)
                    }
                    OrderSide::Sell => {
                        filled.set_sign_positive(false);
                        size.set_sign_positive(false)
                    }
                }
                let status = match update.status {
                    Status::New | Status::PartiallyFilled => types::OrderStatus::Pending,
                    Status::Canceled | Status::Expired | Status::Filled => {
                        types::OrderStatus::Finished
                    }
                    Status::NewAdl | Status::NewInsurance => types::OrderStatus::Pending,
                };
                let trade_size = update.last_trade_size.abs().normalize();
                let trade = if !trade_size.is_zero() {
                    let mut trade = types::OrderTrade {
                        price: update.last_trade_price,
                        size: if matches!(update.side, OrderSide::Buy) {
                            trade_size
                        } else {
                            -trade_size
                        },
                        fee: Decimal::ZERO,
                        fee_asset: None,
                    };
                    if let Some(asset) = update.fee_asset {
                        trade.fee = -update.fee.normalize();
                        trade.fee_asset = Some(asset);
                    }
                    Some(trade)
                } else {
                    None
                };
                Ok(types::OrderUpdate {
                    ts: crate::types::adaptations::from_timestamp(update.trade_ts)?,
                    order: types::Order {
                        id: types::OrderId::from(client_id),
                        target: types::Place { size, kind },
                        state: types::OrderState {
                            filled,
                            cost: if update.filled_size.is_zero() {
                                Decimal::ONE
                            } else {
                                (update.filled_quote_size / update.filled_size).normalize()
                            },
                            status,
                            fees: HashMap::default(),
                        },
                        trade,
                    },
                })
            }
            OrderUpdateFrame::Options(update) => {
                let kind = match update.order_type {
                    OrderType::Limit => match (update.post_only, update.time_in_force) {
                        (false, TimeInForce::Gtc) => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::GoodTilCancelled,
                        ),
                        (false, TimeInForce::Fok) => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::FillOrKill,
                        ),
                        (false, TimeInForce::Ioc) => types::OrderKind::Limit(
                            update.price.normalize(),
                            types::TimeInForce::ImmediateOrCancel,
                        ),
                        (true, _) => types::OrderKind::PostOnly(update.price.normalize()),
                        other => {
                            return Err(ExchangeError::Other(anyhow!(
                                "unsupported time in force: {other:?}"
                            )));
                        }
                    },
                    OrderType::Market => types::OrderKind::Market,
                    OrderType::LimitMaker => types::OrderKind::PostOnly(update.price.normalize()),
                    other => {
                        return Err(ExchangeError::Other(anyhow!(
                            "unsupported order type: {other:?}"
                        )));
                    }
                };
                let filled = update.filled_size.normalize();
                let cost = if filled.is_zero() {
                    Decimal::ONE
                } else {
                    update
                        .filled_amount
                        .normalize()
                        .checked_div(filled)
                        .ok_or_else(|| {
                            ExchangeError::Other(anyhow::anyhow!(
                                "parse options order: failed to calculate cost"
                            ))
                        })?
                };
                let quote_fee = update.fee.normalize().neg();
                // FIXME: we are assuming that the quote is in USDT.
                const QUOTE: Asset = Asset::USDT;
                let state = types::OrderState {
                    filled,
                    cost,
                    status: update.status.try_into()?,
                    fees: HashMap::from([(QUOTE, quote_fee)]),
                };
                let order = types::Order {
                    id: types::OrderId::from(update.client_id.unwrap_or(update.order_id)),
                    target: types::Place {
                        size: update.size.normalize(),
                        kind,
                    },
                    state,
                    // FIXME: we are not parsing the trades.
                    trade: None,
                };
                Ok(types::OrderUpdate {
                    ts: crate::types::adaptations::from_timestamp(update.update_ts)?,
                    order,
                })
            }
        }
    }
}
