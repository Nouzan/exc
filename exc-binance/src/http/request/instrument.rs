use crate::http::error::RestError;

use super::{Rest, RestEndpoint};

/// Exchange info.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExchangeInfo;

impl Rest for ExchangeInfo {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok("/fapi/v1/exchangeInfo".to_string()),
            RestEndpoint::Spot => Ok("/api/v1/exchangeInfo".to_string()),
        }
    }

    fn to_body(&self, _endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        Ok(hyper::Body::empty())
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}
