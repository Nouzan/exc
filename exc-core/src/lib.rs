//! Exc-core: Abstractions for exchanges (the low-level apis).

#![deny(missing_docs)]

/// Core services.
pub use exc_service as service;

/// The definition of an exchange.
pub mod exchange;

/// Transport utils.
pub mod transport;

/// The core types for exchange APIs.
pub use exc_types as types;

#[cfg(feature = "retry")]
/// Retry utils.
pub use exc_service::retry;

/// Utils for creating [`ExcService`](exc_service::ExcService).
pub mod util;

/// Exc Symbol.
pub use exc_symbol as symbol;

pub use self::service::{Adaptor, Exc, ExcLayer, ExcService, ExcServiceExt, IntoExc, Request};
pub use exc_service::{error::InstrumentError, ExchangeError};
pub use positions::prelude::{Asset, Instrument, ParseAssetError, ParseSymbolError, Str, Symbol};
