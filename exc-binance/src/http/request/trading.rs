use rust_decimal::Decimal;
use serde::Serialize;

use crate::types::trading::{OrderSide, OrderType, PositionSide, TimeInForce};

use super::{Rest, RestEndpoint, RestError};

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
    pub time_in_force: TimeInForce,
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

impl Rest for PlaceOrder {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::POST)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/order")),
            _ => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "{endpoint}"
            ))),
        }
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn need_sign(&self) -> bool {
        true
    }

    fn serialize(&self) -> Result<serde_json::Value, RestError> {
        Ok(serde_json::to_value(self)?)
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}

/// Cancel order.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrder {
    /// Symbol.
    pub symbol: String,
    /// Order Id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    /// Client Id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
}

impl Rest for CancelOrder {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::DELETE)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/order")),
            _ => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "{endpoint}"
            ))),
        }
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn need_sign(&self) -> bool {
        true
    }

    fn serialize(&self) -> Result<serde_json::Value, RestError> {
        Ok(serde_json::to_value(self)?)
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}
