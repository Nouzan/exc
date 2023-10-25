pub use crate::core::types::instrument::InstrumentMeta;
use crate::{core::Symbol, Request};
use either::Either;
use exc_core::Str;
use rust_decimal::Decimal;

/// Get instrument request.
#[derive(Debug, Clone)]
pub struct GetInstrument {
    /// Symbol.
    pub symbol: Either<Symbol, Str>,
}

impl Request for GetInstrument {
    type Response = Option<InstrumentMeta<Decimal>>;
}

impl GetInstrument {
    /// Get insturment with the given symbol.
    pub fn with_symbol(symbol: &Symbol) -> Self {
        Self {
            symbol: Either::Left(symbol.clone()),
        }
    }

    /// Get instrument with the given native (exchange-defined) name.
    pub fn with_name(name: &str) -> Self {
        Self {
            symbol: Either::Right(Str::new(name)),
        }
    }
}
