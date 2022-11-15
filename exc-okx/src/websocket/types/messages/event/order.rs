use std::collections::HashMap;

use exc_core::types::{self, Order, OrderUpdate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};
use time::OffsetDateTime;

use crate::error::OkxError;

/// Okx Order State.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Cancelled.
    Canceled,
    /// Live.
    Live,
    /// Partially filled.
    PartiallyFilled,
    /// Filled.
    Filled,
}

/// Okx Order Type.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Market.
    Market,
    /// Limit.
    Limit,
    /// Post only.
    PostOnly,
    /// Fill-or-kill.
    Fok,
    /// Immediate-or-cancel.
    Ioc,
    /// Optimal-limit-ioc
    OptimalLimitIoc,
}

/// Okx Order Side.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    /// Buy.
    Buy,
    /// Sell.
    Sell,
}

/// Okx Order message from websocket channel.
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OkxOrder {
    /// Instrument type.
    pub inst_type: String,
    /// Instrument name.
    pub inst_id: String,
    /// Margin currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub ccy: Option<String>,
    /// Order id.
    pub ord_id: String,
    /// Client id.
    #[serde_as(as = "NoneAsEmptyString")]
    pub cl_ord_id: Option<String>,
    /// Tag.
    #[serde_as(as = "NoneAsEmptyString")]
    pub tag: Option<String>,
    /// Price.
    #[serde_as(as = "NoneAsEmptyString")]
    pub px: Option<Decimal>,
    /// Size.
    pub sz: Decimal,
    /// Notional usd.
    #[serde_as(as = "NoneAsEmptyString")]
    pub notional_usd: Option<Decimal>,
    /// Order type.
    pub ord_type: OrderType,
    /// Side.
    pub side: OrderSide,
    /// Position side.
    #[serde_as(as = "NoneAsEmptyString")]
    pub pos_side: Option<String>,
    /// Trade mode.
    pub td_mode: String,
    /// "Tgt" currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub tgt_ccy: Option<String>,
    /// Fill price.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_px: Option<Decimal>,
    /// Trade id.
    #[serde_as(as = "NoneAsEmptyString")]
    pub trade_id: Option<String>,
    /// Fill size.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_sz: Option<Decimal>,
    /// Fill time.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    #[serde(rename = "fillTime")]
    pub fill_ts: Option<OffsetDateTime>,
    /// Fill fee.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_fee: Option<Decimal>,
    /// Fill fee currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_fee_ccy: Option<String>,
    /// Execute type.
    #[serde_as(as = "NoneAsEmptyString")]
    pub exec_type: Option<String>,
    /// Total filled size.
    pub acc_fill_sz: Decimal,
    /// Filled usd.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_notional_usd: Option<Decimal>,
    /// Average price.
    pub avg_px: Decimal,
    /// State.
    pub state: State,
    /// Leverage.
    #[serde_as(as = "NoneAsEmptyString")]
    pub lever: Option<Decimal>,
    /// Take-profit trigger price.
    #[serde(rename = "tpTriggerPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub take_profit_trigger_price: Option<Decimal>,
    /// Take-profit trigger type.
    #[serde(rename = "tpTriggerPxType")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub take_profit_trigger_type: Option<String>,
    /// Take-profit price.
    #[serde(rename = "tpOrdPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub take_profit_price: Option<Decimal>,
    /// Stop-loss trigger price.
    #[serde(rename = "slTriggerPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_trigger_price: Option<Decimal>,
    /// Stop-loss trigger type.
    #[serde(rename = "slTriggerPxType")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_trigger_type: Option<String>,
    /// Stop-loss price.
    #[serde(rename = "slOrdPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_price: Option<Decimal>,
    /// Fee currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fee_ccy: Option<String>,
    /// Fee.
    pub fee: Decimal,
    /// Rebate currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub rebate_ccy: Option<String>,
    /// Rebate.
    pub rebate: Decimal,
    /// Profit and loss.
    pub pnl: Decimal,
    /// Source.
    #[serde_as(as = "NoneAsEmptyString")]
    pub source: Option<i64>,
    /// Cancel source.
    #[serde_as(as = "NoneAsEmptyString")]
    pub cancel_source: Option<i64>,
    /// Category.
    pub category: String,
    /// Update time.
    #[serde(with = "crate::utils::timestamp_serde")]
    #[serde(rename = "uTime")]
    pub update_ts: OffsetDateTime,
    /// Create time.
    #[serde(with = "crate::utils::timestamp_serde")]
    #[serde(rename = "cTime")]
    pub create_ts: OffsetDateTime,
    /// Request id.
    #[serde_as(as = "NoneAsEmptyString")]
    pub req_id: Option<String>,
    /// Amend result.
    #[serde_as(as = "NoneAsEmptyString")]
    pub amend_result: Option<i64>,
    /// Reduce only.
    pub reduce_only: String,
    /// Code.
    #[serde_as(as = "DisplayFromStr")]
    pub code: i64,
    /// Message.
    pub msg: String,
}

impl TryFrom<OkxOrder> for OrderUpdate {
    type Error = OkxError;

    fn try_from(order: OkxOrder) -> Result<Self, Self::Error> {
        let kind = match order.ord_type {
            OrderType::Market => types::OrderKind::Market,
            OrderType::PostOnly => types::OrderKind::PostOnly(
                order
                    .px
                    .ok_or_else(|| OkxError::parsing_order("missing `px` for post-only order"))?,
            ),
            OrderType::Limit => types::OrderKind::Limit(
                order
                    .px
                    .ok_or_else(|| OkxError::parsing_order("missing `px` for limit order"))?,
                types::TimeInForce::GoodTilCancelled,
            ),
            OrderType::Fok => types::OrderKind::Limit(
                order
                    .px
                    .ok_or_else(|| OkxError::parsing_order("missing `px` for limit order"))?,
                types::TimeInForce::FillOrKill,
            ),
            OrderType::Ioc | OrderType::OptimalLimitIoc => types::OrderKind::Limit(
                order
                    .px
                    .ok_or_else(|| OkxError::parsing_order("missing `px` for limit order"))?,
                types::TimeInForce::ImmediateOrCancel,
            ),
        };
        let status = match order.state {
            State::Canceled | State::Filled => types::OrderStatus::Finished,
            State::Live | State::PartiallyFilled => types::OrderStatus::Pending,
        };
        let mut fees = HashMap::default();
        if let (fee, Some(fee_asset)) = (order.fee, order.fee_ccy) {
            fees.insert(fee_asset, fee);
        }
        if let (rebate, Some(rebate_asset)) = (order.rebate, order.rebate_ccy) {
            fees.insert(rebate_asset, rebate);
        }
        let trade = 'trade: {
            let (Some(price), Some(size), Some(fee), Some(fee_asset)) = (order.fill_px, order.fill_sz, order.fill_fee, order.fill_fee_ccy) else {
                break 'trade None;
            };
            Some(types::OrderTrade {
                price,
                size,
                fee,
                fee_asset: Some(fee_asset),
            })
        };
        Ok(Self {
            ts: order.update_ts,
            order: Order {
                id: types::OrderId::from(order.ord_id),
                target: types::Place {
                    size: if matches!(order.side, OrderSide::Buy) {
                        order.sz.abs().normalize()
                    } else {
                        -(order.sz.abs().normalize())
                    },
                    kind,
                },
                state: types::OrderState {
                    filled: if matches!(order.side, OrderSide::Buy) {
                        order.acc_fill_sz.abs().normalize()
                    } else {
                        -(order.acc_fill_sz.abs().normalize())
                    },
                    cost: if order.acc_fill_sz.is_zero() {
                        Decimal::ONE
                    } else {
                        order.avg_px
                    },
                    status,
                    fees,
                },
                trade,
            },
        })
    }
}
