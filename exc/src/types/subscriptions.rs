use futures::stream::BoxStream;

use crate::ExchangeError;

use super::ticker::Ticker;
use super::Request;

/// Ticker Stream.
pub type TickerStream = BoxStream<'static, Result<Ticker, ExchangeError>>;

/// Subscribe tickers.
#[derive(Debug, Clone)]
pub struct SubscribeTickers {
    /// Instrument.
    pub instrument: String,
}

impl SubscribeTickers {
    /// Create a new [`SubscribeTickers`]
    pub fn new(inst: &str) -> Self {
        Self {
            instrument: inst.to_string(),
        }
    }
}

impl Request for SubscribeTickers {
    type Response = TickerStream;
}
