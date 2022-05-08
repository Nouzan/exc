use std::collections::BTreeMap;

use exc::{
    types::{
        instrument::{InstrumentMeta, SubscribeInstruments},
        Adaptor,
    },
    ExchangeError,
};
use futures::{future::ready, stream::iter, StreamExt};

use super::{
    types::messages::{event::OkxInstrumentMeta, Args},
    Request, Response,
};

impl Adaptor<SubscribeInstruments> for Request {
    fn from_request(req: SubscribeInstruments) -> Result<Self, exc::ExchangeError>
    where
        Self: Sized,
    {
        let tag = req.tag;
        Ok(Self::subscribe(Args(BTreeMap::from([
            ("channel".to_string(), "instruments".to_string()),
            ("instType".to_string(), tag),
        ]))))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<SubscribeInstruments as exc::types::Request>::Response, ExchangeError> {
        match resp {
            Response::Error(err) => Err(ExchangeError::Other(anyhow::anyhow!("status: {err}"))),
            Response::Streaming(stream) => {
                let stream = stream
                    .skip(1)
                    .filter_map(|frame| {
                        ready(match frame {
                            Ok(frame) => frame.into_change().map(Ok),
                            Err(err) => Some(Err(err)),
                        })
                    })
                    .flat_map(|change| match change {
                        Ok(change) => iter(change.deserialize_data::<OkxInstrumentMeta>())
                            .filter_map(|m| match m {
                                Ok(m) => ready(Some(Ok(InstrumentMeta::from(m)))),
                                Err(err) => {
                                    error!("deserialize instrument meta error: {err}, skipped.");
                                    ready(None)
                                }
                            })
                            .left_stream(),
                        Err(err) => {
                            futures::stream::once(
                                async move { Err(ExchangeError::Other(err.into())) },
                            )
                            .right_stream()
                        }
                    })
                    .boxed();
                Ok(stream)
            }
        }
    }
}
