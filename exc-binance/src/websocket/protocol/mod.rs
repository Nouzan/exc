use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use self::stream::{MultiplexRequest, MultiplexResponse};

use super::error::WsError;
use super::request::WsRequest;
use super::response::WsResponse;
use exc::transport::websocket::WsStream;
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
        keep_alive_timeout: Duration,
        default_stream_timeout: Duration,
    ) -> (Self, Arc<stream::Shared>) {
        let transport = keep_alive::layer(
            websocket.sink_map_err(WsError::from).map_err(WsError::from),
            keep_alive_timeout,
        );
        let transport = frame::layer(transport);
        let (transport, state) = stream::layer(transport, default_stream_timeout);
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
        r.id
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
    state: Arc<stream::Shared>,
    svc: Multiplex<Protocol, WsError, Req>,
}

impl WsClient {
    /// Create a [`BinanceWsApi`] using the given websocket stream.
    pub fn with_websocket(
        websocket: WsStream,
        keep_alive_timeout: Duration,
        default_stream_timeout: Duration,
    ) -> Result<Self, WsError> {
        let (protocol, state) =
            Protocol::new(websocket, keep_alive_timeout, default_stream_timeout);
        let svc = Multiplex::with_error_handler(protocol, |err| {
            tracing::error!("protocol error: {err}");
        });
        Ok(Self { svc, state })
    }
}

impl Service<WsRequest> for WsClient {
    type Response = WsResponse;
    type Error = WsError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Err(err) = futures::ready!(self.state.poll_ready(cx)) {
            return Poll::Ready(Err(err));
        }
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: WsRequest) -> Self::Future {
        let is_stream = req.stream;
        self.svc
            .call(req.into())
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
}
