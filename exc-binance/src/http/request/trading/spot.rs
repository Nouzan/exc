use exc_core::types;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    http::error::RestError,
    types::trading::{OrderSide, OrderType, TimeInForce},
};

/// Responsee type.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RespType {
    /// Ack.
    Ack,
    /// Result.
    Result,
    /// Full.
    Full,
}

/// Side effect (for margin)
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SideEffect {
    /// No side effect.
    NoSideEffect,
    /// Margin buy.
    MarginBuy,
    /// Auto repay.
    AutoRepay,
}

/// Place order.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrder {
    /// Symbol.
    pub symbol: String,
    /// Side.
    pub side: OrderSide,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Time-In-Force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// Quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Decimal>,
    /// Quote quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<Decimal>,
    /// Price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// Side effect (for margin).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side_effect_type: Option<SideEffect>,
    /// Client id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    /// New order response type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<RespType>,
}

impl<'a> TryFrom<&'a exc_core::types::PlaceOrder> for PlaceOrder {
    type Error = RestError;

    fn try_from(req: &'a exc_core::types::PlaceOrder) -> Result<Self, Self::Error> {
        let place = req.place;
        let side = if place.size.is_zero() {
            return Err(RestError::PlaceZeroSize);
        } else if place.size.is_sign_positive() {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };
        let (order_type, price, tif) = match place.kind {
            types::OrderKind::Market => (OrderType::Market, None, None),
            types::OrderKind::Limit(price, tif) => {
                let tif = match tif {
                    types::TimeInForce::GoodTilCancelled => Some(TimeInForce::Gtc),
                    types::TimeInForce::FillOrKill => Some(TimeInForce::Fok),
                    types::TimeInForce::ImmediateOrCancel => Some(TimeInForce::Ioc),
                };
                (OrderType::Limit, Some(price), tif)
            }
            types::OrderKind::PostOnly(price) => (OrderType::LimitMaker, Some(price), None),
        };
        Ok(Self {
            symbol: req.instrument.to_uppercase(),
            side,
            order_type,
            quantity: Some(place.size.abs()),
            quote_order_qty: None,
            price,
            new_client_order_id: req.client_id.clone(),
            time_in_force: tif,
            side_effect_type: None,
            new_order_resp_type: Some(RespType::Ack),
        })
    }
}
