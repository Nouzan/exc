use super::{ticker::Ticker, Request};
use crate::ExchangeError;
use futures::stream::BoxStream;

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
    type Response = BoxStream<'static, Result<Ticker, ExchangeError>>;
}
