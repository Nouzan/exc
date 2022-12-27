pub use self::service::{Market, MarketLayer};

/// The market service.
pub mod service;

/// The request type of [`MarketService`](super::MarketService).
pub mod request;

/// The response type of [`MarketService`](super::MarketService).
pub mod response;
