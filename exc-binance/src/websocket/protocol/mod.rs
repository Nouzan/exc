use std::{
    collections::HashSet,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use self::{
    frame::Name,
    stream::{MultiplexRequest, MultiplexResponse},
};

use super::response::WsResponse;
use super::{connect::BinanceWsHost, request::WsRequest};
use super::{error::WsError, request::RequestKind};
use exc_core::transport::websocket::WsStream;
use futures::{future::BoxFuture, FutureExt, Sink, SinkExt, Stream, TryFutureExt, TryStreamExt};
use tokio_tower::multiplex::{Client as Multiplex, TagStore};
use tower::Service;

/// Multiplex protocol.
pub mod stream;

/// Frame protocol.
pub mod frame;

/// Keep-alive protocol.
pub mod keep_alive;

type Req = MultiplexRequest;
type Resp = MultiplexResponse;

trait Transport: Sink<Req, Error = WsError> + Stream<Item = Result<Resp, WsError>> {}

impl<T> Transport for T
where
    T: Sink<Req, Error = WsError>,
    T: Stream<Item = Result<Resp, WsError>>,
{
}

type BoxTransport = Pin<Box<dyn Transport + Send>>;
type Refresh = BoxFuture<'static, ()>;

pin_project_lite::pin_project! {
    /// Binance websocket protocol.
    pub struct Protocol {
        #[pin]
        transport: BoxTransport,
        next_stream_id: usize,
    }
}

impl Protocol {
    fn new(
        websocket: WsStream,
        main_stream: HashSet<Name>,
        keep_alive_timeout: Duration,
        default_stream_timeout: Duration,
        refresh: Option<Refresh>,
    ) -> (Self, Arc<stream::Shared>) {
        let transport = keep_alive::layer(
            websocket.sink_map_err(WsError::from).map_err(WsError::from),
            keep_alive_timeout,
        );
        let transport = frame::layer(transport);
        let (transport, state) =
            stream::layer(transport, main_stream, default_stream_timeout, refresh);
        (
            Self {
                transport: Box::pin(transport),
                next_stream_id: 1,
            },
            state,
        )
    }
}

impl Sink<Req> for Protocol {
    type Error = WsError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().transport.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Req) -> Result<(), Self::Error> {
        self.project().transport.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().transport.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().transport.poll_close(cx)
    }
}

impl Stream for Protocol {
    type Item = Result<Resp, WsError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().transport.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.transport.size_hint()
    }
}

impl TagStore<Req, Resp> for Protocol {
    type Tag = usize;

    fn assign_tag(self: Pin<&mut Self>, r: &mut Req) -> Self::Tag {
        let this = self.project();
        let id = *this.next_stream_id;
        *this.next_stream_id += 1;
        r.id = id;
        id
    }

    fn finish_tag(self: Pin<&mut Self>, r: &Resp) -> Self::Tag {
        match r {
            Resp::MainStream(id, _) => *id,
            Resp::SubStream { id, .. } => *id,
        }
    }
}

impl From<tokio_tower::Error<Protocol, Req>> for WsError {
    fn from(err: tokio_tower::Error<Protocol, Req>) -> Self {
        match err {
            tokio_tower::Error::BrokenTransportSend(err)
            | tokio_tower::Error::BrokenTransportRecv(Some(err)) => err,
            err => Self::TokioTower(err.into()),
        }
    }
}

/// Binance websocket service.
pub struct WsClient {
    endpoint: BinanceWsHost,
    state: Arc<stream::Shared>,
    svc: Multiplex<Protocol, WsError, Req>,
    reconnect: bool,
}

impl WsClient {
    /// Create a [`WsClient`] using the given websocket stream.
    pub fn with_websocket(
        endpoint: BinanceWsHost,
        websocket: WsStream,
        main_stream: HashSet<Name>,
        keep_alive_timeout: Duration,
        default_stream_timeout: Duration,
        refresh: Option<Refresh>,
    ) -> Result<Self, WsError> {
        let (protocol, state) = Protocol::new(
            websocket,
            main_stream,
            keep_alive_timeout,
            default_stream_timeout,
            refresh,
        );
        let shared = state.clone();
        let svc = Multiplex::with_error_handler(protocol, move |err| {
            shared.waker.wake();
            tracing::error!("protocol error: {err}");
        });
        Ok(Self {
            endpoint,
            svc,
            state,
            reconnect: false,
        })
    }

    fn dispatch(&self, req: WsRequest) -> WsRequest {
        tracing::trace!(
            "ws client; dispatching request with endpoint: {:?}",
            self.endpoint,
        );
        match &req.inner {
            RequestKind::DispatchTrades(trades) => match self.endpoint {
                BinanceWsHost::EuropeanOptions => {
                    WsRequest::subscribe(Name::trade(&trades.instrument))
                }
                _ => WsRequest::subscribe(Name::agg_trade(&trades.instrument)),
            },
            RequestKind::DispatchBidAsk(bid_ask) => match self.endpoint {
                BinanceWsHost::EuropeanOptions => {
                    WsRequest::subscribe(Name::depth(&bid_ask.instrument, "10", "100ms"))
                }
                _ => WsRequest::subscribe(Name::book_ticker(&bid_ask.instrument)),
            },
            _ => {
                tracing::error!("ws client; not a dispatch request");
                req
            }
        }
    }
}

impl Service<WsRequest> for WsClient {
    type Response = WsResponse;
    type Error = WsError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.reconnect {
            Poll::Ready(Err(WsError::TransportIsBoken))
        } else {
            self.state.waker.register(cx.waker());
            self.svc.poll_ready(cx)
        }
    }

    fn call(&mut self, mut req: WsRequest) -> Self::Future {
        let is_stream = req.stream;
        let mut dispatched = false;
        loop {
            match req.inner {
                RequestKind::Multiplex(req) => {
                    return self
                        .svc
                        .call(req)
                        .and_then(move |resp| {
                            let resp: WsResponse = resp.into();
                            if is_stream {
                                resp.stream().left_future()
                            } else {
                                futures::future::ready(Ok(resp)).right_future()
                            }
                        })
                        .boxed()
                }
                RequestKind::Reconnect => {
                    self.reconnect = true;
                    return futures::future::ready(Ok(WsResponse::Reconnected)).boxed();
                }
                _ => {
                    if dispatched {
                        break;
                    }
                    req = self.dispatch(req);
                    dispatched = true;
                }
            }
        }
        tracing::error!("ws client; failed to dispatch request");
        futures::future::ready(Err(WsError::TransportIsBoken)).boxed()
    }
}
