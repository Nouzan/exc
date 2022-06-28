use std::collections::BTreeMap;

use exc_core::{
    types::{
        instrument::{InstrumentMeta, SubscribeInstruments},
        trading::{CancelOrder, OrderId, PlaceOrder},
    },
    Adaptor, ExchangeError,
};
use futures::{future::ready, stream::iter, FutureExt, StreamExt};

use crate::error::OkxError;

use super::{
    types::{
        messages::{
            event::{Event, OkxInstrumentMeta, TradeResponse},
            Args,
        },
        response::StatusKind,
    },
    Request, Response,
};

impl Adaptor<SubscribeInstruments> for Request {
    fn from_request(req: SubscribeInstruments) -> Result<Self, exc_core::ExchangeError>
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
    ) -> Result<<SubscribeInstruments as exc_core::Request>::Response, ExchangeError> {
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

impl Adaptor<PlaceOrder> for Request {
    fn from_request(req: PlaceOrder) -> Result<Self, ExchangeError>
    where
        Self: Sized,
    {
        Ok(Self::order(&req.instrument, &req.place))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<PlaceOrder as exc_core::Request>::Response, ExchangeError> {
        let resp = resp.into_unary().map_err(OkxError::Api)?;

        Ok(async move {
            let event = resp.await?.inner;
            let id = if let Event::TradeResponse(TradeResponse::Order {
                code,
                msg,
                mut data,
                ..
            }) = event
            {
                if code == "0" {
                    if let Some(data) = data.pop() {
                        Ok(OrderId::from(data.ord_id))
                    } else {
                        Err(OkxError::Api(StatusKind::EmptyResponse))
                    }
                } else if let Some(data) = data.pop() {
                    Err(OkxError::Api(StatusKind::Other(anyhow::anyhow!(
                        "code={} msg={}",
                        data.s_code,
                        data.s_msg
                    ))))
                } else {
                    Err(OkxError::Api(StatusKind::Other(anyhow::anyhow!(
                        "code={code} msg={msg}"
                    ))))
                }
            } else {
                Err(OkxError::UnexpectedDataType(anyhow::anyhow!("{event:?}")))
            }?;
            Ok(id)
        }
        .boxed())
    }
}

impl Adaptor<CancelOrder> for Request {
    fn from_request(req: CancelOrder) -> Result<Self, ExchangeError>
    where
        Self: Sized,
    {
        Ok(Self::cancel_order(&req.instrument, req.id.as_str()))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<CancelOrder as exc_core::Request>::Response, ExchangeError> {
        let resp = resp.into_unary().map_err(OkxError::Api)?;

        Ok(async move {
            let event = resp.await?.inner;
            if let Event::TradeResponse(TradeResponse::CancelOrder {
                code,
                msg,
                mut data,
                ..
            }) = event
            {
                if code == "0" {
                    if let Some(_data) = data.pop() {
                        Ok(())
                    } else {
                        Err(OkxError::Api(StatusKind::EmptyResponse))
                    }
                } else if let Some(data) = data.pop() {
                    Err(OkxError::Api(StatusKind::Other(anyhow::anyhow!(
                        "code={} msg={}",
                        data.s_code,
                        data.s_msg
                    ))))
                } else {
                    Err(OkxError::Api(StatusKind::Other(anyhow::anyhow!(
                        "code={code} msg={msg}"
                    ))))
                }
            } else {
                Err(OkxError::UnexpectedDataType(anyhow::anyhow!("{event:?}")))
            }?;
            Ok(())
        }
        .boxed())
    }
}
