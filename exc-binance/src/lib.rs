//! Exc-binance: Binance exchange services.

#![deny(missing_docs)]

/// Error.
pub mod error;

/// Rest API support.
pub mod http;

/// Websocket API support.
pub mod websocket;

/// Types.
pub mod types;

/// Endpoint.
pub mod endpoint;

/// Service.
pub mod service;

pub use self::error::Error;
pub use self::service::Binance;
pub use self::types::{request::Request, response::Response};

#[macro_use]
extern crate anyhow;
