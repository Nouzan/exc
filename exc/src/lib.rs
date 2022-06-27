//! Exc: Abstractions for exchanges.

#![deny(missing_docs)]

pub use exc_core::*;

/// Service.
pub mod service;

pub use service::{
    fetch_candles::{FetchCandlesBackward, FetchCandlesBackwardLayer, FetchCandlesService},
    subscribe_tickers::SubscribeTickersService,
};

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
