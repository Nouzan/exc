use serde::Deserialize;

/// Candle.
pub mod candle;

pub use candle::Candle;

/// OKX HTTP API Response.
#[derive(Debug, Deserialize)]
pub struct HttpResponse {
    /// Code.
    pub code: String,
    /// Message.
    pub msg: String,
    /// Data.
    #[serde(default)]
    pub data: Vec<ResponseData>,
}

/// Response data types.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseData {
    /// Candle.
    Candle(Candle),
    /// Placeholder.
    Placeholder,
}
