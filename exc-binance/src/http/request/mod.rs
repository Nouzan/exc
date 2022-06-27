use std::fmt;

use http::{Method, Request};

use super::error::RestError;

/// Utils.
pub mod utils;

/// Instrument.
pub mod instrument;

/// Candle.
pub mod candle;

pub use self::{
    candle::{Interval, QueryCandles},
    instrument::ExchangeInfo,
};

/// Rest payload.
pub trait Rest: Send + Sync + 'static {
    /// Get request method.
    fn method(&self, endpoint: &RestEndpoint) -> Result<Method, RestError>;

    /// Get request path.
    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError>;

    /// Get request body.
    fn to_body(&self, endpoint: &RestEndpoint) -> Result<hyper::Body, RestError>;
}

/// Payload.
pub struct Payload {
    inner: Box<dyn Rest>,
}

impl Payload {
    /// Create a payload from a [`Rest`].
    pub fn new<T>(inner: T) -> Self
    where
        T: Rest,
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

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        self.inner.to_path(endpoint)
    }

    fn to_body(&self, endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        self.inner.to_body(endpoint)
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

impl fmt::Display for RestEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UsdMarginFutures => write!(f, "binance-u"),
            Self::Spot => write!(f, "binance-s"),
        }
    }
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

/// Binance rest requests.
#[derive(Debug, Clone)]
pub struct RestRequest<T> {
    payload: T,
}

impl RestRequest<Payload> {
    /// Create a rest request with given payload.
    pub fn with_payload<T>(payload: T) -> Self
    where
        T: Rest,
    {
        Self::from(Payload::new(payload))
    }
}

impl<T: Rest> RestRequest<T> {
    pub(crate) fn to_http(
        &self,
        endpoint: &RestEndpoint,
    ) -> Result<Request<hyper::Body>, RestError> {
        let uri = format!("{}{}", endpoint.host(), self.payload.to_path(endpoint)?);
        let request = Request::builder()
            .method(self.payload.method(endpoint)?)
            .uri(uri)
            .body(self.payload.to_body(endpoint)?)?;
        Ok(request)
    }
}

impl<T: Rest> From<T> for RestRequest<T> {
    fn from(payload: T) -> Self {
        Self { payload }
    }
}

#[cfg(test)]
mod test {
    use tower::ServiceExt;

    use crate::{
        http::{request, response},
        Binance, Request,
    };

    #[tokio::test]
    async fn test_exchange_info() -> anyhow::Result<()> {
        let api = Binance::usd_margin_futures().connect();
        let resp = api
            .oneshot(Request::with_rest_payload(request::ExchangeInfo))
            .await?
            .into_response::<response::ExchangeInfo>()?;
        println!("{:?}", resp);
        Ok(())
    }

    #[tokio::test]
    async fn test_candle() -> anyhow::Result<()> {
        let api = Binance::usd_margin_futures().connect();
        let resp = api
            .oneshot(Request::with_rest_payload(request::QueryCandles {
                symbol: "btcbusd".to_string(),
                interval: request::Interval::M1,
                start_time: None,
                end_time: None,
                limit: None,
            }))
            .await?
            .into_response::<response::Candles>()?;
        for c in resp {
            println!("{c:?}");
        }
        Ok(())
    }
}
