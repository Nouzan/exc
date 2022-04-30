//! Exc-okx: Okx exchange services.

#![deny(missing_docs)]

/// Websocket Service.
pub mod websocket;

/// All errors.
pub mod error;

/// Util
pub mod util;

#[macro_use]
extern crate tracing;
