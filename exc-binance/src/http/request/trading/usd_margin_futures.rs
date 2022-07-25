use exc_core::types;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    http::{
        error::RestError,
        request::{Payload, Rest, RestEndpoint},
    },
    types::trading::{OrderSide, OrderType, PositionSide, TimeInForce},
};

/// Responsee type.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RespType {
    /// Ack.
    Ack,
    /// Result.
    Result,
}

/// Place order.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrder {
    /// Symbol.
    pub symbol: String,
    /// Side.
    pub side: OrderSide,
    /// Position side.
    pub position_side: PositionSide,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Reduce only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Decimal>,
    /// Price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// Client id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    /// Stop price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Decimal>,
    /// Close position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_position: Option<bool>,
    /// Activation price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_price: Option<Decimal>,
    /// Callback rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_rate: Option<Decimal>,
    /// Time-In-Force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// Working type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_type: Option<String>,
    /// Price protect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_protect: Option<String>,
    /// New order response type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<RespType>,
}

impl<'a> TryFrom<&'a types::PlaceOrder> for PlaceOrder {
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
            types::OrderKind::PostOnly(price) => {
                (OrderType::Limit, Some(price), Some(TimeInForce::Gtx))
            }
        };
        Ok(Self {
            symbol: req.instrument.to_uppercase(),
            side,
            position_side: PositionSide::Both,
            order_type,
            reduce_only: None,
            quantity: Some(place.size.abs()),
            price,
            new_client_order_id: req.client_id.clone(),
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            time_in_force: tif,
            working_type: None,
            price_protect: None,
            new_order_resp_type: None,
        })
    }
}

impl Rest for PlaceOrder {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::POST)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/order")),
            _ => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "only support usd-margin futures"
            ))),
        }
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn need_sign(&self) -> bool {
        true
    }

    fn serialize(&self, _endpoint: &RestEndpoint) -> Result<serde_json::Value, RestError> {
        Ok(serde_json::to_value(self)?)
    }

    fn to_payload(&self) -> Payload {
        Payload::new(self.clone())
    }
}
