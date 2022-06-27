use super::{Rest, RestEndpoint, RestError};

/// Ping USD-M Futures API.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ping;

impl Rest for Ping {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        match endpoint {
            RestEndpoint::Spot => Ok("/api/v1/ping".to_string()),
            RestEndpoint::UsdMarginFutures => Ok("/fapi/v1/ping".to_string()),
        }
    }

    fn to_body(&self, _endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        Ok(hyper::Body::empty())
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}
