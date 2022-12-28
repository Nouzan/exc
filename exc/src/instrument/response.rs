use std::sync::Arc;

use exc_core::{types::instrument::InstrumentMeta, ExchangeError};
use rust_decimal::Decimal;

/// The response type of [`Instruments`](super::Instruments).
pub struct InstrumentsResponse {
    kind: Kind,
}

impl InstrumentsResponse {
    fn new(kind: Kind) -> Self {
        Self { kind }
    }
}

pub(crate) enum Kind {
    Instrument(Option<Arc<InstrumentMeta<Decimal>>>),
}

impl From<Option<Arc<InstrumentMeta<Decimal>>>> for InstrumentsResponse {
    fn from(res: Option<Arc<InstrumentMeta<Decimal>>>) -> Self {
        Self::new(Kind::Instrument(res))
    }
}

impl TryFrom<InstrumentsResponse> for Option<Arc<InstrumentMeta<Decimal>>> {
    type Error = ExchangeError;

    fn try_from(resp: InstrumentsResponse) -> Result<Self, Self::Error> {
        let Kind::Instrument(resp) = resp.kind;
        // else {
        //     return Err(ExchangeError::unexpected_response_type("expecting `Instrument`"));
        // }
        Ok(resp)
    }
}
