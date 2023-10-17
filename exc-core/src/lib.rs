//! Exc-core: Abstractions for exchanges (the low-level apis).

#![deny(missing_docs)]

/// Core services.
pub mod service;

/// Transport utils.
pub mod transport;

/// The core types for exchange APIs.
pub mod types;

/// Errors.
pub mod error;

#[cfg(feature = "retry")]
/// Retry utils.
pub mod retry;

/// Other utils.
pub mod util;

/// Exc Symbol.
pub mod symbol {
    pub use exc_symbol::*;
}

pub use self::error::ExchangeError;
pub use self::service::{Adaptor, Exc, ExcLayer, ExcService, IntoExc, Request};
pub use positions::prelude::{Asset, Instrument, ParseAssetError, ParseSymbolError, Str, Symbol};
