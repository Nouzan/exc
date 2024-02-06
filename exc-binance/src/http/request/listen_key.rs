use serde::Serialize;

use super::{Rest, RestEndpoint, RestError};

/// Get current listen key.
#[derive(Debug, Clone, Copy, Default)]
pub struct CurrentListenKey;

impl Rest for CurrentListenKey {
    fn method(&self, _endpoint: &super::RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::POST)
    }

    fn to_path(&self, endpoint: &super::RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok("/fapi/v1/listenKey".to_string()),
            RestEndpoint::EuropeanOptions => Ok("/eapi/v1/listenKey".to_string()),
            RestEndpoint::Spot(options) => {
                if options.margin.is_some() {
                    Ok("/sapi/v1/userDataStream".to_string())
                } else {
                    Ok("/api/v3/userDataStream".to_string())
                }
            }
        }
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(*self)
    }
}

/// Delete current listen key.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteListenKey {
    /// Listen key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_key: Option<String>,
}

impl Rest for DeleteListenKey {
    fn method(&self, _endpoint: &super::RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::DELETE)
    }

    fn to_path(&self, endpoint: &super::RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok("/fapi/v1/listenKey".to_string()),
            RestEndpoint::EuropeanOptions => Ok("/eapi/v1/listenKey".to_string()),
            RestEndpoint::Spot(options) => {
                if options.margin.is_some() {
                    Ok("/sapi/v1/userDataStream".to_string())
                } else {
                    Ok("/api/v3/userDataStream".to_string())
                }
            }
        }
    }

    fn serialize(&self, _endpoint: &RestEndpoint) -> Result<serde_json::Value, RestError> {
        Ok(serde_json::to_value(self)?)
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}
