//! Exc: Abstractions for exchanges.

#![deny(missing_docs)]

/// Subscribe tickers.
pub mod subscribe_tickers;

/// Trade.
pub mod trade;

/// Book.
pub mod book;

/// Subscribe instruments.
pub mod instrument;

/// Fetch candles.
pub mod fetch_candles;

/// Trading service.
pub mod trading;

pub use self::{
    fetch_candles::{FetchCandlesBackward, FetchCandlesBackwardLayer, FetchCandlesService},
    subscribe_tickers::SubscribeTickersService,
};
pub use exc_core::*;

#[cfg(feature = "okx")]
/// Okx exchange service.
pub mod okx {
    pub use exc_okx::*;
}

#[cfg(feature = "binance")]
/// Binance exchange service.
pub mod binance {
    pub use exc_binance::*;
}
