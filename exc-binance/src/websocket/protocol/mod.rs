use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use super::error::WsError;
use super::request::WsRequest;
use super::response::WsResponse;
use exc::transport::websocket::WsStream;
use futures::{future::BoxFuture, Sink, SinkExt, Stream, TryStreamExt};
use tokio_tower::multiplex::{Client as Multiplex, TagStore};
use tower::Service;

/// Multiplex protocol.
pub mod stream;

/// Frame protocol.
pub mod frame;

/// Keep-alive protocol.
pub mod keep_alive;

type Req = WsRequest;
type Resp = WsResponse;

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
    fn new(websocket: WsStream, timeout: Duration) -> (Self, Arc<stream::Shared>) {
        let transport = keep_alive::layer(
            websocket.sink_map_err(WsError::from).map_err(WsError::from),
            timeout,
        );
        let transport = frame::layer(transport);
        let (transport, state) = stream::layer(transport);
        let transport = transport
            .with_flat_map(|req: Req| futures::stream::once(futures::future::ready(Ok(req.into()))))
            .and_then(|resp| futures::future::ready(Ok(resp.into())));
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
        r.inner.id = id;
        id
    }

    fn finish_tag(self: Pin<&mut Self>, r: &Resp) -> Self::Tag {
        r.inner.id
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

/// Binance websocket api service.
pub struct BinanceWsApi {
    state: Arc<stream::Shared>,
    svc: Multiplex<Protocol, WsError, Req>,
}

impl BinanceWsApi {
    /// Create a [`BinanceWsApi`] using the given websocket stream.
    pub fn with_websocket(websocket: WsStream, timeout: Duration) -> Result<Self, WsError> {
        let (protocol, state) = Protocol::new(websocket, timeout);
        let svc = Multiplex::with_error_handler(protocol, |err| {
            tracing::error!("protocol error: {err}");
        });
        Ok(Self { svc, state })
    }
}

impl Service<Req> for BinanceWsApi {
    type Response = Resp;
    type Error = WsError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Err(err) = futures::ready!(self.state.poll_ready(cx)) {
            return Poll::Ready(Err(err));
        }
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.svc.call(req)
    }
}
