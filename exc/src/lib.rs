//! Exc: Abstractions for exchanges.

#![deny(missing_docs)]

/// Exc services.
pub mod services;

/// Util.
pub mod util;

pub use exc_core::*;
pub use services::{
    book::SubscribeBidAskService,
    fetch_candles::{
        FetchCandlesBackward, FetchCandlesBackwardLayer, FetchCandlesForward,
        FetchCandlesForwardLayer, FetchCandlesService,
    },
    instrument::{FetchInstrumentsService, SubscribeInstrumentsService},
    subscribe_tickers::{SubscribeTickersService, TradeBidAsk, TradeBidAskServiceLayer},
    trade::SubscribeTradesService,
    trading::{CheckOrderService, SubscribeOrdersService, TradingService},
};
pub use util::ExcExt;

/// Prelude.
pub mod prelude {
    pub use crate::services::{
        book::SubscribeBidAskService,
        fetch_candles::FetchCandlesService,
        instrument::{FetchInstrumentsService, SubscribeInstrumentsService},
        subscribe_tickers::SubscribeTickersService,
        trade::SubscribeTradesService,
        trading::{CheckOrderService, SubscribeOrdersService, TradingService},
        utils::ReconnectService,
    };
    pub use crate::types::{Period, Place, PlaceOrderOptions};
    pub use crate::util::ExcExt;

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
