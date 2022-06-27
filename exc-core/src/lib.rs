//! Exc-core: Abstractions for exchanges (the core part).

#![deny(missing_docs)]

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

pub use self::types::{Adaptor, Request};
pub use error::ExchangeError;
pub use exchange::{
    service::{Exc, ExcMut},
    Exchange, ExchangeLayer, ExchangeService,
};
