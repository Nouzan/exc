use exc_core::types::instrument::{FetchInstruments, InstrumentMeta};
use exc_core::{Adaptor, ExchangeError, Str};
use serde::Deserialize;

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
        #[derive(Debug, Deserialize, Default)]
        struct Params {
            family: Option<Str>,
            uly: Option<Str>,
        }

        let (tag, params) = req
            .tag
            .split_once('?')
            .map(|(ty, params)| serde_qs::from_str::<Params>(params).map(|p| (Str::new(ty), p)))
            .transpose()
            .map_err(anyhow::Error::from)?
            .unwrap_or_else(|| (req.tag, Params::default()));

        let req = Self::Get(Get::Instruments(Instruments {
            inst_type: tag,
            inst_family: params.family,
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
