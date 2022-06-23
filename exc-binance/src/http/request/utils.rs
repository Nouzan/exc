use super::{Rest, RestEndpoint, RestError};

/// Ping USD-M Futures API.
#[derive(Debug, Clone, Copy, Default)]
pub struct UsdMPing;

impl Rest for UsdMPing {
    fn endpoint(&self) -> RestEndpoint {
        super::RestEndpoint::UsdMarginFutures
    }

    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn path(&self) -> &str {
        "/fapi/v1/ping"
    }

    fn body(&self) -> Result<hyper::Body, RestError> {
        Ok(hyper::Body::empty())
    }
}
