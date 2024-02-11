use exc_core::types;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::{
    http::{
        error::RestError,
        request::{Payload, Rest, RestEndpoint},
    },
    types::trading::{OrderSide, TimeInForce},
};

use super::RespType;

/// Supported order types for european options.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Limit.
    Limit,
    /// Market.
    Market,
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
    /// Reduce only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Post only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Quantity.
    pub quantity: Decimal,
    /// Price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// Client id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    /// Time-In-Force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// New order response type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<RespType>,
    /// Is the order a MMP order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mmp: Option<bool>,
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
        let (order_type, price, tif, post_only) = match place.kind {
            types::OrderKind::Market => (OrderType::Market, None, None, None),
            types::OrderKind::Limit(price, tif) => {
                let tif = match tif {
                    types::TimeInForce::GoodTilCancelled => Some(TimeInForce::Gtc),
                    types::TimeInForce::FillOrKill => Some(TimeInForce::Fok),
                    types::TimeInForce::ImmediateOrCancel => Some(TimeInForce::Ioc),
                };
                (OrderType::Limit, Some(price), tif, None)
            }
            types::OrderKind::PostOnly(price) => (
                OrderType::Limit,
                Some(price),
                Some(TimeInForce::Gtc),
                Some(true),
            ),
        };
        Ok(Self {
            symbol: req.opts.instrument().to_uppercase(),
            side,
            order_type,
            reduce_only: None,
            quantity: place.size.abs(),
            price,
            client_order_id: req.opts.client_id().map(|s| s.to_string()),
            time_in_force: tif,
            new_order_resp_type: None,
            post_only,
            is_mmp: None,
        })
    }
}

impl Rest for PlaceOrder {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::POST)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::EuropeanOptions => Ok("/eapi/v1/order".to_string()),
            _ => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "only support european options"
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
