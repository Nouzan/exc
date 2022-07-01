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
    pub symbol: String,
    /// Client id.
    #[serde(rename = "c")]
    pub client_id: String,
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
    pub fee_asset: Option<String>,
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

impl Nameable for AccountEvent {
    fn to_name(&self) -> Name {
        match self {
            Self::ListenKeyExpired { .. } => Name::listen_key_expired(),
            Self::OrderTradeUpdate { order, .. } => {
                Name::order_trade_update(&order.symbol.to_lowercase())
            }
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

impl TryFrom<StreamFrame> for OrderUpdate {
    type Error = WsError;

    fn try_from(frame: StreamFrame) -> Result<Self, Self::Error> {
        if let StreamFrameKind::AccountEvent(e) = frame.data {
            if let AccountEvent::OrderTradeUpdate { order, .. } = e {
                Ok(order)
            } else {
                Err(WsError::UnexpectedFrame(anyhow::anyhow!("{e:?}")))
            }
        } else {
            Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}")))
        }
    }
}
