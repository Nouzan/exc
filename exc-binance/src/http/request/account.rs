use serde::Serialize;

use crate::http::error::RestError;

use super::{Rest, RestEndpoint};

/// List sub-accounts.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListSubAccounts {
    /// Email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Is freezed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_freeze: Option<bool>,
    /// Page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    /// Limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

impl Rest for ListSubAccounts {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "`ListSubAccounts` only available on `binance-s`"
            ))),
            RestEndpoint::Spot(_options) => Ok("/sapi/v1/sub-account/list".to_string()),
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

/// Assets of a sub-account (in the spot account).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubAccountAssets {
    /// Email.
    pub email: String,
}

impl Rest for GetSubAccountAssets {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "`GetSubAccountAssets` only available on `binance-s`"
            ))),
            RestEndpoint::Spot(_options) => Ok("/sapi/v3/sub-account/assets".to_string()),
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

/// Get details of margin account of a sub-account.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubAccountMargin {
    /// Email.
    pub email: String,
}

impl Rest for GetSubAccountMargin {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "`GetSubAccountMargin` only available on `binance-s`"
            ))),
            RestEndpoint::Spot(_options) => Ok("/sapi/v1/sub-account/margin/account".to_string()),
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

/// Get details of margin account of a sub-account.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubAccountFutures {
    /// Email.
    pub email: String,
    /// Type.
    pub futures_type: usize,
}

impl GetSubAccountFutures {
    /// USD-Margin future's account.
    pub fn usd(email: &str) -> Self {
        Self {
            email: email.to_string(),
            futures_type: 1,
        }
    }
    /// Coin-Margin future's account.
    pub fn coin(email: &str) -> Self {
        Self {
            email: email.to_string(),
            futures_type: 2,
        }
    }
}

impl Rest for GetSubAccountFutures {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "`GetSubAccountMargin` only available on `binance-s`"
            ))),
            RestEndpoint::Spot(_options) => Ok("/sapi/v2/sub-account/futures/account".to_string()),
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
