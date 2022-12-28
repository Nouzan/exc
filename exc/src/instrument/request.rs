use crate::core::Symbol;
use crate::types::instrument::GetInstrument;

use super::response::InstrumentsResponse;

/// The request type of [`Instruments`](super::Instruments).
#[derive(Debug, Clone)]
pub struct InstrumentsRequest {
    kind: Kind,
}

impl InstrumentsRequest {
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

impl From<GetInstrument> for InstrumentsRequest {
    fn from(req: GetInstrument) -> Self {
        Self::new(Kind::GetInstrument(req))
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Kind {
    GetInstrument(GetInstrument),
}

impl InstrumentsRequest {
    pub(crate) fn kind(&self) -> &Kind {
        &self.kind
    }
}

impl crate::Request for InstrumentsRequest {
    type Response = InstrumentsResponse;
}
