use std::collections::BTreeMap;

use exc_core::{
    types::{
        instrument::{InstrumentMeta, SubscribeInstruments},
        trading::{CancelOrder, OrderId, PlaceOrder},
        BidAsk, Canceled, OrderUpdate, Placed, SubscribeBidAsk, SubscribeOrders, SubscribeTrades,
        Trade,
    },
    Adaptor, ExchangeError, Str,
};
use futures::{future::ready, stream::iter, FutureExt, StreamExt, TryStreamExt};
use time::OffsetDateTime;

use crate::error::OkxError;

use super::{
    types::{
        messages::{
            event::{order::OkxOrder, Event, OkxInstrumentMeta, TradeResponse},
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
            (Str::new_inline("channel"), Str::new_inline("instruments")),
            (Str::new_inline("instType"), tag),
        ]))))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<SubscribeInstruments as exc_core::Request>::Response, ExchangeError> {
        match resp {
            Response::Error(err) => Err(ExchangeError::Other(anyhow::anyhow!("status: {err}"))),
            Response::Reconnected => Err(ExchangeError::Other(anyhow::anyhow!(
                "invalid response kind"
            ))),
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
                                Ok(m) => ready(Some(
                                    InstrumentMeta::try_from(m).map_err(ExchangeError::from),
                                )),
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

impl Adaptor<SubscribeOrders> for Request {
    fn from_request(req: SubscribeOrders) -> Result<Self, exc_core::ExchangeError>
    where
        Self: Sized,
    {
        Ok(Self::subscribe(Args::subscribe_orders(&req.instrument)))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<SubscribeOrders as exc_core::Request>::Response, ExchangeError> {
        match resp {
            Response::Error(err) => Err(ExchangeError::Other(anyhow::anyhow!("status: {err}"))),
            Response::Reconnected => Err(ExchangeError::Other(anyhow::anyhow!(
                "invalid response kind"
            ))),
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
                        Ok(change) => iter(change.deserialize_data::<OkxOrder>())
                            .filter_map(|m| {
                                match m.map_err(OkxError::from).and_then(OrderUpdate::try_from) {
                                    Ok(m) => ready(Some(Ok(m))),
                                    Err(err) => {
                                        error!(%err, "deserialize order error, skipped.");
                                        ready(None)
                                    }
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
        Ok(Self::order(&req))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<PlaceOrder as exc_core::Request>::Response, ExchangeError> {
        let resp = resp.into_unary().map_err(OkxError::Api)?;

        Ok(async move {
            let event = resp.await?.inner;
            let (ts, id) = if let Event::TradeResponse(TradeResponse::Order {
                code,
                msg,
                mut data,
                ..
            }) = event
            {
                if code == "0" {
                    if let Some(data) = data.pop() {
                        #[cfg(not(feature = "prefer-client-id"))]
                        {
                            let id = OrderId::from(data.ord_id);
                            Ok((OffsetDateTime::now_utc(), id))
                        }
                        #[cfg(feature = "prefer-client-id")]
                        if let Some(id) = if data.cl_ord_id.is_empty() {
                            None
                        } else {
                            Some(data.cl_ord_id)
                        } {
                            Ok((OffsetDateTime::now_utc(), OrderId::from(id)))
                        } else {
                            Err(OkxError::MissingClientId)
                        }
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
            Ok(Placed {
                id,
                order: None,
                ts,
            })
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
                        Ok(Canceled {
                            ts: OffsetDateTime::now_utc(),
                            order: None,
                        })
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
            Ok(Canceled {
                ts: OffsetDateTime::now_utc(),
                order: None,
            })
        }
        .boxed())
    }
}

impl Adaptor<SubscribeTrades> for Request {
    fn from_request(req: SubscribeTrades) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe_trades(&req.instrument))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<SubscribeTrades as exc_core::Request>::Response, ExchangeError> {
        match resp {
            Response::Streaming(stream) => {
                let stream = stream
                    .skip(1)
                    .flat_map(|frame| {
                        let res: Result<Vec<Result<Trade, OkxError>>, OkxError> =
                            frame.and_then(|f| f.inner.try_into());
                        match res {
                            Ok(tickers) => futures::stream::iter(tickers).left_stream(),
                            Err(err) => {
                                futures::stream::once(async move { Err(err) }).right_stream()
                            }
                        }
                    })
                    .map_err(ExchangeError::from)
                    .boxed();
                Ok(stream)
            }
            Response::Error(status) => Err(OkxError::Api(status).into()),
            Response::Reconnected => Err(ExchangeError::Other(anyhow::anyhow!(
                "invalid response kind"
            ))),
        }
    }
}

impl Adaptor<SubscribeBidAsk> for Request {
    fn from_request(req: SubscribeBidAsk) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe_bid_ask(&req.instrument))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<SubscribeBidAsk as exc_core::Request>::Response, ExchangeError> {
        match resp {
            Response::Streaming(stream) => {
                let stream = stream
                    .skip(1)
                    .flat_map(|frame| {
                        let res: Result<Vec<Result<BidAsk, OkxError>>, OkxError> =
                            frame.and_then(|f| f.inner.try_into());
                        match res {
                            Ok(tickers) => futures::stream::iter(tickers).left_stream(),
                            Err(err) => {
                                futures::stream::once(async move { Err(err) }).right_stream()
                            }
                        }
                    })
                    .map_err(ExchangeError::from)
                    .boxed();
                Ok(stream)
            }
            Response::Error(status) => Err(OkxError::Api(status).into()),
            Response::Reconnected => Err(ExchangeError::Other(anyhow::anyhow!(
                "invalid response kind"
            ))),
        }
    }
}
