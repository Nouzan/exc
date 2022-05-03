//! Exc-okx: Okx exchange services.

#![deny(missing_docs)]

/// Websocket API.
pub mod websocket;

/// Http API.
pub mod http;

/// All errors.
pub mod error;

/// Util
pub mod util;

#[macro_use]
extern crate tracing;
