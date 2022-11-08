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

/// Account.
pub mod account;

pub use self::{
    account::{GetSubAccountAssets, ListSubAccounts},
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

    /// Whether need sign.
    fn need_sign(&self) -> bool {
        false
    }

    /// Serialize.
    fn serialize(&self, _endpoint: &RestEndpoint) -> Result<serde_json::Value, RestError> {
        Ok(serde_json::json!({}))
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

    fn need_sign(&self) -> bool {
        self.inner.need_sign()
    }

    fn serialize(&self, endpoint: &RestEndpoint) -> Result<serde_json::Value, RestError> {
        self.inner.serialize(endpoint)
    }

    fn to_payload(&self) -> Payload {
        self.clone()
    }
}

/// Margin mode.
#[derive(Debug, Clone, Copy)]
pub enum MarginOp {
    /// Loan.
    Loan,
    /// Repay.
    Repay,
}

/// Margin options.
#[derive(Debug, Clone, Copy)]
pub struct MarginOptions {
    /// Buy.
    pub buy: Option<MarginOp>,
    /// Sell.
    pub sell: Option<MarginOp>,
}

/// Spot options.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpotOptions {
    /// Enable margin.
    pub margin: Option<MarginOptions>,
}

impl SpotOptions {
    /// With margin.
    pub fn with_margin(buy: Option<MarginOp>, sell: Option<MarginOp>) -> Self {
        Self {
            margin: Some(MarginOptions { buy, sell }),
        }
    }
}

/// Binance rest api endpoints.
#[derive(Debug, Clone, Copy)]
pub enum RestEndpoint {
    /// USD-M Futures.
    UsdMarginFutures,
    /// Spot.
    /// Set it to `true` to enable margin trading.
    Spot(SpotOptions),
}

impl fmt::Display for RestEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UsdMarginFutures => write!(f, "binance-u"),
            Self::Spot(_) => write!(f, "binance-s"),
        }
    }
}

impl RestEndpoint {
    /// Get host.
    pub fn host(&self) -> &'static str {
        match self {
            Self::UsdMarginFutures => "https://fapi.binance.com",
            Self::Spot(_) => "https://api.binance.com",
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
        let mut uri = format!("{}{}", endpoint.host(), self.payload.to_path(endpoint)?);
        tracing::trace!("building http request: uri={uri}");
        let value = self.payload.serialize(endpoint)?;
        let body = if self.payload.need_sign() {
            if let Some(key) = key.as_ref() {
                let value = key.sign(value)?;
                let s = serde_urlencoded::to_string(&value)?;
                tracing::trace!("params: {s}");
                match self.payload.method(endpoint)? {
                    http::Method::GET => {
                        // FIXME: this is too dirty.
                        uri.push('?');
                        uri.push_str(&s);
                        hyper::Body::empty()
                    }
                    _ => hyper::Body::from(s),
                }
            } else {
                return Err(RestError::NeedApikey);
            }
        } else {
            hyper::Body::from(serde_urlencoded::to_string(&value)?)
        };
        let mut request = Request::builder()
            .method(self.payload.method(endpoint)?)
            .uri(uri)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body)?;
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
        types::key::BinanceKey,
        Binance, Request,
    };

    async fn do_test_exchange_info(api: Binance) -> anyhow::Result<()> {
        let resp = api
            .oneshot(Request::with_rest_payload(request::ExchangeInfo))
            .await?
            .into_response::<response::ExchangeInfo>()?;
        println!("{:?}", resp);
        Ok(())
    }

    async fn do_test_candle(api: Binance, inst: &str) -> anyhow::Result<()> {
        let resp = api
            .oneshot(Request::with_rest_payload(request::QueryCandles {
                symbol: inst.to_uppercase(),
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

    async fn do_test_listen_key(api: Binance) -> anyhow::Result<()> {
        let listen_key = api
            .oneshot(Request::with_rest_payload(request::CurrentListenKey))
            .await?
            .into_response::<response::ListenKey>()?;
        println!("{listen_key}");
        Ok(())
    }

    async fn do_test_delete_listen_key(
        api: Binance,
        listen_key: Option<String>,
    ) -> anyhow::Result<()> {
        let listen_key = api
            .oneshot(Request::with_rest_payload(request::DeleteListenKey {
                listen_key,
            }))
            .await?
            .into_response::<response::Unknown>()?;
        println!("{listen_key:?}");
        Ok(())
    }

    async fn do_test_list_sub_accounts(api: Binance) -> anyhow::Result<response::SubAccounts> {
        let sub_accounts = api
            .oneshot(Request::with_rest_payload(
                request::ListSubAccounts::default(),
            ))
            .await?
            .into_response::<response::SubAccounts>()?;
        println!("{sub_accounts:?}");
        Ok(sub_accounts)
    }

    async fn do_test_get_sub_account_assets(api: Binance, email: &str) -> anyhow::Result<()> {
        let assets = api
            .oneshot(Request::with_rest_payload(request::GetSubAccountAssets {
                email: email.to_string(),
            }))
            .await?
            .into_response::<response::SubAccountBalances>()?;
        println!("{assets:?}");
        Ok(())
    }

    #[tokio::test]
    async fn test_exchange_info() -> anyhow::Result<()> {
        let apis = [
            Binance::usd_margin_futures().connect(),
            Binance::spot().connect(),
        ];
        for api in apis {
            do_test_exchange_info(api).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_candle() -> anyhow::Result<()> {
        let apis = [
            (Binance::usd_margin_futures().connect(), "btcbusd"),
            (Binance::spot().connect(), "btcusdt"),
        ];
        for (api, inst) in apis {
            do_test_candle(api, inst).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_listen_key() -> anyhow::Result<()> {
        if let Ok(key) = var("BINANCE_KEY") {
            let key = serde_json::from_str::<BinanceKey>(&key)?;
            let apis = [
                Binance::usd_margin_futures().private(key.clone()).connect(),
                Binance::spot().private(key).connect(),
            ];
            for api in apis {
                do_test_listen_key(api).await?;
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_listen_key() -> anyhow::Result<()> {
        if let Ok(key) = var("BINANCE_KEY") {
            let key = serde_json::from_str::<BinanceKey>(&key)?;
            let apis = [
                (
                    Binance::usd_margin_futures().private(key.clone()).connect(),
                    false,
                ),
                (Binance::spot().private(key).connect(), true),
            ];
            for (mut api, listen_key) in apis {
                let listen_key = if listen_key {
                    let listen_key = (&mut api)
                        .oneshot(Request::with_rest_payload(request::CurrentListenKey))
                        .await?
                        .into_response::<response::ListenKey>()?;
                    Some(listen_key.to_string())
                } else {
                    None
                };
                do_test_delete_listen_key(api, listen_key).await?;
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_list_sub_accounts() -> anyhow::Result<()> {
        if let Ok(key) = var("BINANCE_MAIN") {
            let key = serde_json::from_str::<BinanceKey>(&key)?;
            let api = Binance::spot().private(key).connect();
            do_test_list_sub_accounts(api).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_sub_account_assets() -> anyhow::Result<()> {
        if let Ok(key) = var("BINANCE_MAIN") {
            let key = serde_json::from_str::<BinanceKey>(&key)?;
            let api = Binance::spot().private(key).connect();
            let sub_accounts = do_test_list_sub_accounts(api.clone()).await?;
            for account in sub_accounts.sub_accounts {
                do_test_get_sub_account_assets(api.clone(), &account.email).await?;
            }
        }
        Ok(())
    }
}
