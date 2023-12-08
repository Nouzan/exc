//! Exc-binance: Binance exchange services.

#![deny(missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(any(feature = "rustls-tls", feature = "native-tls"))] {
        #[macro_use]
        extern crate anyhow;

        /// Types.
        pub mod types;

        /// Error.
        pub mod error;

        /// Websocket API support.
        pub mod websocket;

        /// Rest API support.
        pub mod http;

        /// Endpoint.
        pub mod endpoint;

        /// Service.
        pub mod service;

        /// Exchange.
        pub mod exchange;

        pub use self::service::Binance;
        pub use self::error::Error;
        pub use self::http::request::{MarginOp, SpotOptions};
        pub use self::types::{request::Request, response::Response};
    } else {
        compile_error!("Either feature 'rustls-tls' or 'native-tls' must be enabled");
    }
}
