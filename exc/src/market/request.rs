use crate::core::Symbol;
use crate::types::instrument::GetInstrument;

use super::response::Response;

/// The request type of [`MarketService`](super::MarketService).
#[derive(Debug, Clone)]
pub struct Request {
    kind: Kind,
}

impl Request {
    fn new(kind: Kind) -> Self {
        Self { kind }
    }

    /// Get instrument.
    pub fn get_instrument(symbol: &Symbol) -> Self {
        Self::from(GetInstrument::with_symbol(symbol))
    }

    /// Get instrument with native (exchange-defined) name.
    pub fn get_instrument_with_native_name(name: &str) -> Self {
        Self::from(GetInstrument::with_name(name))
    }
}

impl From<GetInstrument> for Request {
    fn from(req: GetInstrument) -> Self {
        Self::new(Kind::GetInstrument(req))
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Kind {
    GetInstrument(GetInstrument),
}

impl Request {
    pub(crate) fn kind(&self) -> &Kind {
        &self.kind
    }
}

impl crate::Request for Request {
    type Response = Response;
}
