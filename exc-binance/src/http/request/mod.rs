use http::{Method, Request};

use super::error::RestError;

/// Utils.
pub mod utils;

/// Instrument.
pub mod instrument;

pub use self::instrument::ExchangeInfo;

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
    fn method(&self, endpoint: &RestEndpoint) -> Result<Method, RestError> {
        self.inner.method(endpoint)
    }

    fn path(&self, endpoint: &RestEndpoint) -> Result<&str, RestError> {
        self.inner.path(endpoint)
    }

    fn body(&self, endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        self.inner.body(endpoint)
    }
}

/// Binance rest api endpoints.
#[derive(Debug, Clone, Copy)]
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
    /// Get request method.
    fn method(&self, endpoint: &RestEndpoint) -> Result<Method, RestError>;

    /// Get request path.
    fn path(&self, endpoint: &RestEndpoint) -> Result<&str, RestError>;

    /// Get request body.
    fn body(&self, endpoint: &RestEndpoint) -> Result<hyper::Body, RestError>;
}

/// Binance rest requests.
#[derive(Debug, Clone)]
pub struct RestRequest<T> {
    payload: T,
}

impl<T: Rest> RestRequest<T> {
    pub(crate) fn to_http(
        &self,
        endpoint: &RestEndpoint,
    ) -> Result<Request<hyper::Body>, RestError> {
        let uri = format!("{}{}", endpoint.host(), self.payload.path(endpoint)?);
        let request = Request::builder()
            .method(self.payload.method(endpoint)?)
            .uri(uri)
            .body(self.payload.body(endpoint)?)?;
        Ok(request)
    }
}

impl<T: Rest> From<T> for RestRequest<T> {
    fn from(payload: T) -> Self {
        Self { payload }
    }
}
