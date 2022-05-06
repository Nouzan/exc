//! Exc: Abstractions for exchanges.

#![deny(missing_docs)]

/// Exchange.
pub mod exchange;

/// Transport.
pub mod transport;

/// Types.
pub mod types;

/// Errors.
pub mod error;

/// Service.
pub mod service;

pub use error::ExchangeError;
pub use exchange::{Exchange, ExchangeLayer};
pub use service::{subscribe_tickers::SubscribeTickersService, fetch_candles::{FetchCandlesService, FetchCandlesBackward, FetchCandlesBackwardLayer}};
