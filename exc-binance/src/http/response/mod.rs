use super::error::RestError;
use anyhow::anyhow;
use exc::ExchangeError;
use http::StatusCode;
use serde::{de::DeserializeOwned, Deserialize};

/// Empty.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Empty {}

/// Binance rest api response data.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Data {
    /// Empty.
    Empty(Empty),
}

/// Binance rest api response.
#[derive(Debug, Clone)]
pub struct RestResponse<T> {
    data: T,
}

impl<T> RestResponse<T> {
    /// Into inner data.
    pub fn into_inner(self) -> T {
        self.data
    }

    pub(crate) async fn from_http(resp: http::Response<hyper::Body>) -> Result<Self, RestError>
    where
        T: DeserializeOwned,
    {
        let status = resp.status();
        tracing::trace!("http response status: {}", resp.status());
        let value = hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(RestError::from)
            .and_then(|bytes| {
                serde_json::from_slice::<serde_json::Value>(&bytes).map_err(RestError::from)
            });
        match status {
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
            _ => match value {
                Ok(value) => match serde_json::from_value::<T>(value) {
                    Ok(data) => Ok(Self { data }),
                    Err(err) => Err(err.into()),
                },
                Err(err) => Err(err),
            },
        }
    }
}
