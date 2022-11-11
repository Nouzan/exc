use super::error::RestError;
use anyhow::anyhow;
use either::Either;
use exc_core::ExchangeError;
use http::StatusCode;
use serde::{de::DeserializeOwned, Deserialize};

/// Instrument.
pub mod instrument;

/// Candle.
pub mod candle;

/// Listen key.
pub mod listen_key;

/// Error message.
pub mod error_message;

/// Trading.
pub mod trading;

/// Account.
pub mod account;

pub use self::{
    account::{
        SubAccountBalances, SubAccountFutures, SubAccountFuturesPositions, SubAccountMargin,
        SubAccounts,
    },
    candle::Candle,
    error_message::ErrorMessage,
    instrument::{ExchangeInfo, SpotExchangeInfo, UFExchangeInfo},
    listen_key::ListenKey,
    trading::Order,
};

/// Candles.
pub type Candles = Vec<Candle>;

/// Unknown response.
pub type Unknown = serde_json::Value;

/// Binance rest api response data.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Data {
    /// Candles.
    Candles(Vec<Candle>),
    /// Exchange info.
    ExchangeInfo(ExchangeInfo),
    /// Listen key.
    ListenKey(ListenKey),
    /// Error Message.
    Error(ErrorMessage),
    /// Order.
    Order(Order),
    /// Sub-accounts.
    SubAccounts(SubAccounts),
    /// Sub-account balances.
    SubAccountBalances(SubAccountBalances),
    /// Sub-account margin.
    SubAccountMargin(SubAccountMargin),
    /// Sub-account futures.
    SubAccountFutures(SubAccountFutures),
    /// Sub-account futures postions.
    SubAccountFuturesPositions(SubAccountFuturesPositions),
    /// Unknwon.
    Unknwon(Unknown),
}

impl TryFrom<Data> for Unknown {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Unknwon(u) => Ok(u),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}

/// Binance rest api response.
#[derive(Debug)]
pub struct RestResponse<T> {
    data: T,
}

impl<T> RestResponse<T> {
    /// Into inner data.
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Convert into a response of the given type.
    pub fn into_response<U>(self) -> Result<U, RestError>
    where
        U: TryFrom<T, Error = RestError>,
    {
        U::try_from(self.into_inner())
    }

    pub(crate) async fn from_http(resp: http::Response<hyper::Body>) -> Result<Self, RestError>
    where
        T: DeserializeOwned,
    {
        let status = resp.status();
        tracing::trace!("http response status: {}", resp.status());
        let bytes = hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(RestError::from);
        let value =
            bytes.and_then(
                |bytes| match serde_json::from_slice::<serde_json::Value>(&bytes) {
                    Ok(value) => {
                        tracing::debug!("resp: {value}");
                        Ok(Either::Left(value))
                    }
                    Err(_) => match std::str::from_utf8(&bytes) {
                        Ok(text) => Ok(Either::Right(text.to_string())),
                        Err(err) => Err(err.into()),
                    },
                },
            );
        let res = match status {
            StatusCode::TOO_MANY_REQUESTS => Err(RestError::Exchange(ExchangeError::RateLimited(
                anyhow!("too many requests"),
            ))),
            StatusCode::IM_A_TEAPOT => Err(RestError::Exchange(ExchangeError::RateLimited(
                anyhow!("I'm a teapot"),
            ))),
            StatusCode::SERVICE_UNAVAILABLE => Err(RestError::Exchange(match value {
                Ok(msg) => ExchangeError::Unavailable(anyhow!("{msg}")),
                Err(err) => ExchangeError::Unavailable(anyhow!("failed to read msg: {err}")),
            })),
            StatusCode::FORBIDDEN => match value {
                Ok(Either::Left(value)) => Err(RestError::Exchange(ExchangeError::Forbidden(
                    anyhow!("{value}"),
                ))),
                Ok(Either::Right(_)) => Err(RestError::Exchange(ExchangeError::Forbidden(
                    anyhow!("no msg"),
                ))),
                Err(err) => Err(RestError::Exchange(ExchangeError::Forbidden(anyhow!(
                    "failed to parse msg: {err}"
                )))),
            },
            _ => match value {
                Ok(Either::Left(value)) => match serde_json::from_value::<T>(value) {
                    Ok(data) => Ok(Self { data }),
                    Err(err) => Err(err.into()),
                },
                Ok(Either::Right(text)) => Err(RestError::Text(text)),
                Err(err) => Err(err),
            },
        };
        tracing::trace!("finished processing http response");
        res
    }
}
