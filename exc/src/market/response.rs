use std::sync::Arc;

use exc_core::{types::instrument::InstrumentMeta, ExchangeError};
use rust_decimal::Decimal;

/// The response type of [`MarketService`](super::MarketService).
pub struct Response {
    kind: Kind,
}

impl Response {
    fn new(kind: Kind) -> Self {
        Self { kind }
    }
}

pub(crate) enum Kind {
    Instrument(Option<Arc<InstrumentMeta<Decimal>>>),
}

impl From<Option<Arc<InstrumentMeta<Decimal>>>> for Response {
    fn from(res: Option<Arc<InstrumentMeta<Decimal>>>) -> Self {
        Self::new(Kind::Instrument(res))
    }
}

impl TryFrom<Response> for Option<Arc<InstrumentMeta<Decimal>>> {
    type Error = ExchangeError;

    fn try_from(resp: Response) -> Result<Self, Self::Error> {
        let Kind::Instrument(resp) = resp.kind;
        // else {
        //     return Err(ExchangeError::unexpected_response_type("expecting `Instrument`"));
        // }
        Ok(resp)
    }
}
