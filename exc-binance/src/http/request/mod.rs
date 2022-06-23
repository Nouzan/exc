use http::{Method, Request};

use super::error::RestError;

/// Utils.
pub mod utils;

/// Payload.
pub struct Payload {
    inner: Box<dyn Rest + Send + Sync + 'static>,
}

impl Payload {
    /// Create a payload from a [`Rest`].
    pub fn new<T>(inner: T) -> Self
    where
        T: Rest + Send + Sync + 'static,
    {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl Rest for Payload {
    fn endpoint(&self) -> RestEndpoint {
        self.inner.endpoint()
    }

    fn method(&self) -> Method {
        self.inner.method()
    }

    fn path(&self) -> &str {
        self.inner.path()
    }

    fn body(&self) -> Result<hyper::Body, RestError> {
        self.inner.body()
    }
}

/// Binance rest api endpoints.
pub enum RestEndpoint {
    /// USD-M Futures.
    UsdMarginFutures,
    /// Spot.
    Spot,
}

impl RestEndpoint {
    /// Get host.
    pub fn host(&self) -> &'static str {
        match self {
            Self::UsdMarginFutures => "https://fapi.binance.com",
            Self::Spot => "https://api.binance.com",
        }
    }
}

/// Rest payload.
pub trait Rest {
    /// Get endpoint.
    fn endpoint(&self) -> RestEndpoint;

    /// Get request method.
    fn method(&self) -> Method;

    /// Get request path.
    fn path(&self) -> &str;

    /// Get request body.
    fn body(&self) -> Result<hyper::Body, RestError>;
}

/// Binance rest requests.
#[derive(Debug, Clone)]
pub struct RestRequest<T> {
    payload: T,
}

impl<T: Rest> RestRequest<T> {
    pub(crate) fn to_http(&self) -> Result<Request<hyper::Body>, RestError> {
        let uri = format!("{}{}", self.payload.endpoint().host(), self.payload.path());
        let request = Request::builder()
            .method(self.payload.method())
            .uri(uri)
            .body(self.payload.body()?)?;
        Ok(request)
    }
}

impl<T: Rest> From<T> for RestRequest<T> {
    fn from(payload: T) -> Self {
        Self { payload }
    }
}
