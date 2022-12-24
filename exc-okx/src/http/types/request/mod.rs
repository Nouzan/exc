use exc_core::ExchangeError;
use http::Request;
use hyper::Body;
use serde::Serialize;

use crate::key::OkxKey as Key;

use self::history_candles::HistoryCandles;
use self::instruments::Instruments;
use self::trading::Order;

/// History candles.
pub mod history_candles;

/// Instruments query.
pub mod instruments;

/// Trading.
pub mod trading;

/// Okx HTTP API request types.
#[derive(Debug, Clone)]
pub enum HttpRequest {
    /// Get (public requests).
    Get(Get),
    /// Private Get.
    PrivateGet(PrivateGet),
}

/// Okx HTTP API get request types.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Get {
    /// History candles.
    HistoryCandles(HistoryCandles),
    /// Get instruments.
    Instruments(Instruments),
}

impl Get {
    pub(crate) fn uri(&self) -> &'static str {
        match self {
            Self::HistoryCandles(_) => "/api/v5/market/history-candles",
            Self::Instruments(_) => "/api/v5/public/instruments",
        }
    }
}

/// Okx HTTP API get request types.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum PrivateGet {
    /// Order.
    Order(Order),
}

impl PrivateGet {
    pub(crate) fn uri(&self) -> &'static str {
        match self {
            Self::Order(_) => "/api/v5/trade/order",
        }
    }

    pub(crate) fn to_request(&self, host: &str, key: &Key) -> Result<Request<Body>, ExchangeError> {
        serde_qs::to_string(self)
            .map_err(|err| ExchangeError::Other(err.into()))
            .and_then(|q| {
                let uri = format!("{}?{q}", self.uri());
                let sign = key
                    .sign_now("GET", &uri, false)
                    .map_err(|e| ExchangeError::KeyError(anyhow::anyhow!("{e}")))?;
                Request::get(format!("{host}{uri}"))
                    .header("OK-ACCESS-KEY", key.apikey.as_str())
                    .header("OK-ACCESS-SIGN", sign.signature.as_str())
                    .header("OK-ACCESS-TIMESTAMP", sign.timestamp.as_str())
                    .header("OK-ACCESS-PASSPHRASE", key.passphrase.as_str())
                    .body(Body::empty())
                    .map_err(|err| ExchangeError::Other(err.into()))
            })
    }
}
