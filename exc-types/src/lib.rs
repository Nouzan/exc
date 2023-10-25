#![deny(missing_docs)]

//! `exc-request`: Define the request and response types that are used in the exchange APIs.

/// Ticker.
pub mod ticker;

/// Trade.
pub mod trade;

/// Book.
pub mod book;

/// Candle.
pub mod candle;

/// Instrument.
pub mod instrument;

/// Trading.
pub mod trading;

/// Utils.
pub mod utils;

/// Exc Symbol.
pub mod symbol {
    pub use exc_symbol::*;
}

pub use self::instrument::{
    FetchInstruments, InstrumentMeta, InstrumentStream, SubscribeInstruments,
};
pub use book::{BidAsk, BidAskStream, SubscribeBidAsk};
pub use candle::{
    Candle, CandleStream, Period, PeriodKind, QueryCandles, QueryFirstCandles, QueryLastCandles,
};
pub use positions::prelude::Str;
pub use ticker::{SubscribeTickers, Ticker, TickerStream};
pub use trade::{SubscribeTrades, Trade, TradeStream};
pub use trading::{
    CancelOrder, Canceled, GetOrder, Order, OrderId, OrderKind, OrderState, OrderStatus,
    OrderStream, OrderTrade, OrderUpdate, Place, PlaceOrder, PlaceOrderOptions, Placed,
    SubscribeOrders, TimeInForce,
};
