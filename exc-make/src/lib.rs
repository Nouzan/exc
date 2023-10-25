#![deny(missing_docs)]

//! `exc-make`: Define what an exchange is with [`MakeService`](tower_make::MakeService)s.

/// Make a service to subscribe instruments.
pub mod instruments;

/// Make a service to subscribe tickers.
pub mod tickers;

/// Make a service to place orders.
pub mod place;

/// Make a service to cancel orders.
pub mod cancel;

/// Make a service to check orders.
pub mod check;

/// Make a service to subscribe order updates.
pub mod orders;

pub use self::{
    cancel::{MakeCancelOrder, MakeCancelOrderOptions},
    check::{MakeCheckOrder, MakeCheckOrderOptions},
    instruments::{MakeInstruments, MakeInstrumentsOptions},
    orders::{MakeSubscribeOrders, MakeSubscribeOrdersOptions},
    place::{MakePlaceOrder, MakePlaceOrderOptions},
    tickers::{MakeTickers, MakeTickersOptions},
};
