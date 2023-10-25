/// Period utils.
pub mod period;

/// Create a service to subscribe tickers from subscribe trades and bid/ask.
pub mod trade_bid_ask;

pub use period::{trunc, PeriodExt};
