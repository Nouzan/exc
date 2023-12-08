//! Exc-okx: Okx exchange services.

#![deny(missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(any(feature = "rustls-tls", feature = "native-tls"))] {
        /// The OKX service of both ws and rest APIs.
        pub mod service;

        /// Exchange.
        pub mod exchange;

        pub use exchange::OkxExchange;
        pub use service::{Okx, OkxRequest, OkxResponse};
    } else {
        compile_error!("Either feature 'rustls-tls' or 'native-tls' must be enabled");
    }
}

/// Utils
pub mod utils;

/// Websocket API.
pub mod websocket;

/// Http API.
pub mod http;

/// All errors.
pub mod error;

/// Key.
pub mod key;

#[macro_use]
extern crate tracing;
