//! Exc: Abstractions for exchanges.

#![deny(missing_docs)]

/// Exc services.
pub mod services;

pub use exc_core::*;
pub use services::{
    book::SubscribeBidAskService,
    fetch_candles::{
        FetchCandlesBackward, FetchCandlesBackwardLayer, FetchCandlesForward,
        FetchCandlesForwardLayer, FetchCandlesService, FetchFirstCandlesService,
    },
    instrument::{FetchInstrumentsService, SubscribeInstrumentsService},
    subscribe_tickers::{
        SubscribeTickersService, TradeBidAsk, TradeBidAskService, TradeBidAskServiceLayer,
    },
    trade::SubscribeTradesService,
    trading::{CheckOrderService, SubscribeOrdersService, TradingService},
};

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
