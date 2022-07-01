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
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/listenKey")),
            _ => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "{endpoint}"
            ))),
        }
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}

/// Delete current listen key.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeleteListenKey;

impl Rest for DeleteListenKey {
    fn method(&self, _endpoint: &super::RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::DELETE)
    }

    fn to_path(&self, endpoint: &super::RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/listenKey")),
            _ => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "{endpoint}"
            ))),
        }
    }

    fn need_apikey(&self) -> bool {
        true
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}
