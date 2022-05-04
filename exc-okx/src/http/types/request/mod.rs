use serde::Serialize;

use self::history_candles::HistoryCandles;

/// History candles.
pub mod history_candles;

/// Okx HTTP API request types.
pub enum HttpRequest {
    /// Get (public requests).
    Get(Get),
}

/// Okx HTTP API get request types.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Get {
    /// History candles.
    HistoryCandles(HistoryCandles),
}

impl Get {
    pub(crate) fn uri(&self) -> &'static str {
        match self {
            Self::HistoryCandles(_) => "/api/v5/market/history-candles",
        }
    }
}
