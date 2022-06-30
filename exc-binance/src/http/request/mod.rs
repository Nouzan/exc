use std::fmt;

use http::{HeaderValue, Method, Request};

use crate::types::key::BinanceKey;

use super::error::RestError;

/// Utils.
pub mod utils;

/// Instrument.
pub mod instrument;

/// Candle.
pub mod candle;

/// Listen key.
pub mod listen_key;

/// Trading.
pub mod trading;

pub use self::{
    candle::{Interval, QueryCandles},
    instrument::ExchangeInfo,
    listen_key::{CurrentListenKey, DeleteListenKey},
};

/// Rest payload.
pub trait Rest: Send + Sync + 'static {
    /// Get request method.
    fn method(&self, endpoint: &RestEndpoint) -> Result<Method, RestError>;

    /// Get request path.
    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError>;

    /// add request header.
    fn add_headers(
        &self,
        _endpoint: &RestEndpoint,
        _headers: &mut hyper::HeaderMap,
    ) -> Result<(), RestError> {
        Ok(())
    }

    /// Whether need apikey.
    fn need_apikey(&self) -> bool {
        false
    }

    /// Get request body.
    fn to_body(&self, _endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        Ok(hyper::Body::empty())
    }

    /// Clone.
    fn to_payload(&self) -> Payload;
}

/// Payload.
pub struct Payload {
    inner: Box<dyn Rest>,
}

impl Clone for Payload {
    fn clone(&self) -> Self {
        self.inner.to_payload()
    }
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

    fn add_headers(
        &self,
        endpoint: &RestEndpoint,
        headers: &mut hyper::HeaderMap,
    ) -> Result<(), RestError> {
        self.inner.add_headers(endpoint, headers)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        self.inner.to_path(endpoint)
    }

    fn need_apikey(&self) -> bool {
        self.inner.need_apikey()
    }

    fn to_body(&self, endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        self.inner.to_body(endpoint)
    }

    fn to_payload(&self) -> Payload {
        self.clone()
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
        key: Option<&BinanceKey>,
    ) -> Result<Request<hyper::Body>, RestError> {
        let uri = format!("{}{}", endpoint.host(), self.payload.to_path(endpoint)?);
        let mut request = Request::builder()
            .method(self.payload.method(endpoint)?)
            .uri(uri)
            .body(self.payload.to_body(endpoint)?)?;
        let headers = request.headers_mut();
        if let Some(key) = key {
            if self.payload.need_apikey() {
                headers.insert("X-MBX-APIKEY", HeaderValue::from_str(&key.apikey)?);
            }
        }
        self.payload.add_headers(endpoint, headers)?;
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
    use std::env::var;
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

    #[tokio::test]
    async fn test_listen_key() -> anyhow::Result<()> {
        if let Ok(key) = var("BINANCE_KEY") {
            let key = serde_json::from_str(&key)?;
            let api = Binance::usd_margin_futures().private(key).connect();
            let listen_key = api
                .oneshot(Request::with_rest_payload(request::CurrentListenKey))
                .await?
                .into_response::<response::ListenKey>()?;
            println!("{listen_key}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_listen_key() -> anyhow::Result<()> {
        if let Ok(key) = var("BINANCE_KEY") {
            let key = serde_json::from_str(&key)?;
            let api = Binance::usd_margin_futures().private(key).connect();
            let listen_key = api
                .oneshot(Request::with_rest_payload(request::DeleteListenKey))
                .await?
                .into_response::<response::Unknown>()?;
            println!("{listen_key:?}");
        }
        Ok(())
    }
}
