//! Exc-core: Abstractions for exchanges (the core part).

#![deny(missing_docs)]

/// Symbol.
pub mod symbol;

/// Exchange.
pub mod exchange;

/// Transport.
pub mod transport;

/// Types.
pub mod types;

/// Errors.
pub mod error;

#[cfg(feature = "retry")]
/// Retry.
pub mod retry;

/// Utils.
pub mod util;

pub use self::error::ExchangeError;
pub use self::exchange::{Adaptor, Exc, ExcLayer, ExcMut, ExcService, IntoExc, Request};
pub use positions::prelude::{Asset, Instrument, ParseAssetError, ParseSymbolError, Str, Symbol};
