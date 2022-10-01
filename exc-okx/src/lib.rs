//! Exc-okx: Okx exchange services.

#![deny(missing_docs)]

/// The OKX service of both ws and rest APIs.
pub mod service;

/// Websocket API.
pub mod websocket;

/// Http API.
pub mod http;

/// All errors.
pub mod error;

/// Key.
pub mod key;

/// Utils
pub mod utils;

pub use service::{Okx, OkxRequest, OkxResponse};

#[macro_use]
extern crate tracing;
