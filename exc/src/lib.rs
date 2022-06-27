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

/// Utils.
pub mod util;

pub use self::types::{Adaptor, Request};
pub use error::ExchangeError;
pub use exchange::{Exchange, ExchangeLayer};
pub use service::{
    fetch_candles::{FetchCandlesBackward, FetchCandlesBackwardLayer, FetchCandlesService},
    subscribe_tickers::SubscribeTickersService,
};
