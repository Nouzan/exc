use exc_core::types::instrument::{FetchInstruments, InstrumentMeta};
use exc_core::Adaptor;

use crate::http::types::request::instruments::Instruments;
use crate::http::types::request::Get;
use crate::http::types::response::ResponseData;
use async_stream::stream;
use futures::StreamExt;

use super::HttpRequest;

impl Adaptor<FetchInstruments> for HttpRequest {
    fn from_request(req: FetchInstruments) -> Result<Self, exc_core::ExchangeError>
    where
        Self: Sized,
    {
        let req = Self::Get(Get::Instruments(Instruments {
            inst_type: req.tag,
            ..Default::default()
        }));
        Ok(req)
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<FetchInstruments as exc_core::Request>::Response, exc_core::ExchangeError> {
        let stream = stream! {
            for data in resp.data {
                trace!("received a data: {data:?}");
                if let ResponseData::Instruments(c) = data {
                    yield Ok(InstrumentMeta::from(c))
                }
            }
        };
        Ok(stream.boxed())
    }
}
