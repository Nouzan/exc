use super::{Rest, RestEndpoint, RestError};

/// Ping USD-M Futures API.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ping;

impl Rest for Ping {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn path(&self, endpoint: &RestEndpoint) -> Result<&str, RestError> {
        match endpoint {
            RestEndpoint::Spot => Ok("/api/v1/ping"),
            RestEndpoint::UsdMarginFutures => Ok("/fapi/v1/ping"),
        }
    }

    fn body(&self, _endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        Ok(hyper::Body::empty())
    }
}
