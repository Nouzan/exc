use super::{Rest, RestEndpoint, RestError};
use serde::Serialize;

/// Usd-Margin futures.
pub mod usd_margin_futures;

/// Spot.
pub mod spot;

/// Place order.
#[derive(Debug, Clone)]
pub struct PlaceOrder {
    pub(crate) inner: exc_core::types::PlaceOrder,
}

impl PlaceOrder {
    fn dispatch(&self, endpoint: &RestEndpoint) -> Result<PlaceOrderKind, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(PlaceOrderKind::UsdMarginFutures(
                usd_margin_futures::PlaceOrder::try_from(&self.inner)?,
            )),
            RestEndpoint::Spot => Ok(PlaceOrderKind::Spot(spot::PlaceOrder::try_from(
                &self.inner,
            )?)),
        }
    }
}

/// Place order kind.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum PlaceOrderKind {
    /// Usd-Margin futures.
    UsdMarginFutures(usd_margin_futures::PlaceOrder),
    /// Spot.
    Spot(spot::PlaceOrder),
}

impl Rest for PlaceOrder {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::POST)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/order")),
            _ => Ok(format!("/api/v3/order")),
        }
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn need_sign(&self) -> bool {
        true
    }

    fn serialize(&self, endpoint: &RestEndpoint) -> Result<serde_json::Value, RestError> {
        Ok(serde_json::to_value(&self.dispatch(endpoint)?)?)
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}

/// Get order inner.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderInner {
    /// Symbol.
    pub symbol: String,
    /// Order Id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    /// Client Id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
}

/// Cancel order.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrder {
    /// Inner.
    #[serde(flatten)]
    pub inner: GetOrderInner,
}

impl Rest for CancelOrder {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::DELETE)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/order")),
            _ => Ok(format!("/api/v3/order")),
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

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}

/// Get order.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrder {
    /// Inner.
    #[serde(flatten)]
    pub inner: GetOrderInner,
}

impl Rest for GetOrder {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/order")),
            _ => Ok(format!("/api/v3/order")),
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

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}
