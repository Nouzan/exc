/// Period utils.
pub mod period;

/// Create a service to subscribe tickers from subscribe trades and bid/ask.
pub mod trade_bid_ask;

/// Create a service to subscribe instruments by first fetching.
pub mod fetch_instruments_first;

/// Poll instruments.
#[cfg(feature = "poll")]
pub mod poll_instruments;

/// Fetch candles.
#[cfg(feature = "fetch-candles")]
pub mod fetch_candles;

pub use period::{trunc, PeriodExt};
