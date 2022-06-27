use anyhow::anyhow;
use exc_core::{error::InstrumentError, ExchangeError};
use serde::Deserialize;

/// Candle.
pub mod candle;

/// Trading.
pub mod trading;

pub use candle::Candle;
pub use trading::OrderDetail;

/// Okx HTTP API Response (with `code` and `msg`).
#[derive(Debug, Deserialize)]
pub struct FullHttpResponse {
    /// Code.
    pub code: String,
    /// Message.
    pub msg: String,
    /// Data.
    #[serde(default)]
    pub data: Vec<ResponseData>,
}

/// Okx HTTP API Response.
#[derive(Debug)]
pub struct HttpResponse {
    /// Data.
    pub data: Vec<ResponseData>,
}

impl TryFrom<FullHttpResponse> for HttpResponse {
    type Error = ExchangeError;

    fn try_from(full: FullHttpResponse) -> Result<Self, Self::Error> {
        let code = full.code;
        let msg = full.msg;
        match code.as_str() {
            "0" => Ok(Self { data: full.data }),
            "51001" => Err(ExchangeError::Instrument(InstrumentError::NotFound)),
            "50011" => Err(ExchangeError::RateLimited(anyhow!("{msg}"))),
            "50013" => Err(ExchangeError::Unavailable(anyhow!("{msg}"))),
            "51603" => Err(ExchangeError::OrderNotFound),
            _ => Err(ExchangeError::Api(anyhow!("code={code} msg={msg}",))),
        }
    }
}

/// Response data types.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseData {
    /// Candle.
    Candle(Candle),
    /// Order.
    Order(Box<OrderDetail>),
}
