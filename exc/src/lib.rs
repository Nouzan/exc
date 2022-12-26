//! Exc: Abstractions for exchanges (high-level apis).

#![deny(missing_docs)]

/// Utils.
pub mod util;

pub use exc_core::*;
pub use util::{
    book::SubscribeBidAskService,
    fetch_candles::FetchCandlesService,
    instrument::{FetchInstrumentsService, SubscribeInstrumentsService},
    subscribe_tickers::SubscribeTickersService,
    trade::SubscribeTradesService,
    trading::{CheckOrderService, SubscribeOrdersService, TradingService},
    ExcExt,
};

/// Prelude.
pub mod prelude {
    pub use crate::types::{Period, Place, PlaceOrderOptions};
    pub use crate::util::{
        book::SubscribeBidAskService,
        fetch_candles::FetchCandlesService,
        instrument::{FetchInstrumentsService, SubscribeInstrumentsService},
        reconnect::ReconnectService,
        subscribe_tickers::SubscribeTickersService,
        trade::SubscribeTradesService,
        trading::{CheckOrderService, SubscribeOrdersService, TradingService},
        ExcExt,
    };

    #[cfg(feature = "okx")]
    pub use crate::Okx;

    #[cfg(feature = "binance")]
    pub use crate::Binance;
}

#[cfg(feature = "okx")]
/// Okx exchange service.
pub mod okx {
    pub use exc_okx::*;
}

#[cfg(feature = "binance")]
/// Binance exchange service.
pub mod binance {
    pub use exc_binance::*;
}

#[cfg(feature = "okx")]
pub use crate::okx::Okx;

#[cfg(feature = "binance")]
pub use crate::binance::Binance;
