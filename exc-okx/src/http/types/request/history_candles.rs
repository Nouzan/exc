use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

/// History Candles.
#[serde_as]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryCandles {
    /// Instrument Id.
    pub inst_id: String,
    /// After (older) (millis).
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub after: Option<u64>,
    /// Before (newer) (millis).
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub before: Option<u64>,
    /// Bar.
    pub bar: Option<&'static str>,
    /// Limit (last).
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub limit: Option<usize>,
}
