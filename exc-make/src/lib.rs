#![deny(missing_docs)]

//! `exc-make`: Define what an exchange is with [`MakeService`](tower_make::MakeService)s.

/// Make a service to subscribe instruments.
pub mod instruments;

/// Make a service to subscribe tickers.
pub mod tickers;

pub use self::{instruments::MakeInstruments, tickers::MakeTickers};
