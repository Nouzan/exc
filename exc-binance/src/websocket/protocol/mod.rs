use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use super::error::WsError;
use super::request::WsRequest;
use super::response::WsResponse;
use exc::transport::websocket::WsStream;
use futures::{Sink, Stream};
use tokio_tower::multiplex::{Client as Multiplex, TagStore};

trait Transport: Sink<WsRequest, Error = WsError> + Stream<Item = Result<WsResponse, WsError>> {}

impl<T> Transport for T
where
    T: Sink<WsRequest, Error = WsError>,
    T: Stream<Item = Result<WsResponse, WsError>>,
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
    fn new(websocket: WsStream, timeout: Duration) -> Self {
        todo!()
    }
}

impl Sink<WsRequest> for Protocol {
    type Error = WsError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().transport.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: WsRequest) -> Result<(), Self::Error> {
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
    type Item = Result<WsResponse, WsError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().transport.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.transport.size_hint()
    }
}

impl TagStore<WsRequest, WsResponse> for Protocol {
    type Tag = usize;

    fn assign_tag(self: Pin<&mut Self>, r: &mut WsRequest) -> Self::Tag {
        let this = self.project();
        let id = *this.next_stream_id;
        *this.next_stream_id += 1;
        r.id = id;
        id
    }

    fn finish_tag(self: Pin<&mut Self>, r: &WsResponse) -> Self::Tag {
        r.id
    }
}

/// Binance websocket api service.
pub struct BinanceWsApi {
    svc: Multiplex<Protocol, WsError, WsRequest>,
}

impl BinanceWsApi {
    pub(crate) fn new(websocket: WsStream, timeout: Duration) -> Result<Self, WsError> {
        let protocol = Protocol::new(websocket, timeout);
        let svc = Multiplex::with_error_handler(protocol, |err| {
            tracing::error!("protocol error: {err}");
        });
        Ok(Self { svc })
    }
}
