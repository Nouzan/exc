use serde::Serialize;

use super::{Rest, RestEndpoint, RestError};

/// Intervals.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum Interval {
    /// 1 minute.
    #[serde(rename = "1m")]
    M1,
    /// 3 minutes.
    #[serde(rename = "3m")]
    M3,
    /// 5 minutes.
    #[serde(rename = "5m")]
    M5,
    /// 15 minutes.
    #[serde(rename = "15m")]
    M15,
    /// 30 mintues.
    #[serde(rename = "30m")]
    M30,
    /// 1 hour.
    #[serde(rename = "1h")]
    H1,
    /// 2 hours.
    #[serde(rename = "2h")]
    H2,
    /// 4 hours.
    #[serde(rename = "4h")]
    H4,
    /// 6 hours.
    #[serde(rename = "6h")]
    H6,
    /// 8 hours.
    #[serde(rename = "8h")]
    H8,
    /// 12 hours.
    #[serde(rename = "12h")]
    H12,
    /// 1 day.
    #[serde(rename = "1d")]
    D1,
    /// 3 days.
    #[serde(rename = "3d")]
    D3,
    /// 1 week.
    #[serde(rename = "1w")]
    W1,
    /// 1 month.
    #[serde(rename = "1M")]
    Mon1,
}

/// Query candles.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryCandles {
    /// Instrument.
    pub symbol: String,
    /// Interval.
    pub interval: Interval,
    /// Start time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// End time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

impl Rest for QueryCandles {
    fn method(&self, _endpoint: &RestEndpoint) -> Result<http::Method, RestError> {
        Ok(http::Method::GET)
    }

    fn to_path(&self, endpoint: &RestEndpoint) -> Result<String, RestError> {
        let qs = serde_qs::to_string(self)?;
        match endpoint {
            RestEndpoint::UsdMarginFutures => Ok(format!("/fapi/v1/klines?{qs}")),
            RestEndpoint::Spot => Err(RestError::UnsupportedEndpoint(anyhow::anyhow!(
                "{endpoint}"
            ))),
        }
    }

    fn to_body(&self, _endpoint: &RestEndpoint) -> Result<hyper::Body, RestError> {
        Ok(hyper::Body::empty())
    }

    fn to_payload(&self) -> super::Payload {
        super::Payload::new(self.clone())
    }
}
