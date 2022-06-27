//! Exc: Abstractions for exchanges.

#![deny(missing_docs)]

pub use exc_core::*;

/// Service.
pub mod service;

pub use service::{
    fetch_candles::{FetchCandlesBackward, FetchCandlesBackwardLayer, FetchCandlesService},
    subscribe_tickers::SubscribeTickersService,
};
