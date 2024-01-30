use exc_core::types::instrument::{FetchInstruments, InstrumentMeta};
use exc_core::{Adaptor, ExchangeError};

use crate::http::types::request::instruments::Instruments;
use crate::http::types::request::Get;
use crate::http::types::response::ResponseData;
use crate::utils::inst_tag::parse_inst_tag;
use async_stream::stream;
use futures::StreamExt;

use super::HttpRequest;

impl Adaptor<FetchInstruments> for HttpRequest {
    fn from_request(req: FetchInstruments) -> Result<Self, exc_core::ExchangeError>
    where
        Self: Sized,
    {
        let (tag, params) = parse_inst_tag(&req.tag)?;
        let req = Self::Get(Get::Instruments(Instruments {
            inst_type: tag,
            inst_family: params.inst_family,
            uly: params.uly,
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
                    yield InstrumentMeta::try_from(c).map_err(ExchangeError::from)
                }
            }
        };
        Ok(stream.boxed())
    }
}
