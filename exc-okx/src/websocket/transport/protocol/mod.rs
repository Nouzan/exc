use crate::websocket::types::{
    request::{ClientStream, Request},
    response::{Response, ServerStream, Status, StatusKind},
};
use atomic_waker::AtomicWaker;
use exc::transport::websocket::WsStream;
use futures::{future::BoxFuture, FutureExt, Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use pin_project_lite::pin_project;
use std::{pin::Pin, sync::Arc};
use std::{
    task::{Context, Poll},
    time::Duration,
};
use thiserror::Error;
use tokio_tower::multiplex::{Client, TagStore};
use tokio_tungstenite::tungstenite::Message;
use tower::Service;

mod frame;
mod message;
mod ping_pong;
mod stream;

pub use frame::FrameError;
pub use message::MessageError;
pub use ping_pong::PingPongError;
pub use stream::StreamingError;

type Req = ClientStream;
type Resp = Result<ServerStream, Status>;

/// Protocol Error.
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// Transport Errors.
    #[error("transport: {0}")]
    Transport(#[from] StreamingError<FrameError<MessageError<PingPongError>>>),

    /// Tokio tower error.
    #[error("tokio-tower: {0}")]
    TokioTower(anyhow::Error),
    // /// Subsribed.
    // #[error("subscribed: {0}")]
    // Subscribed(Args),
}

/// Okx websocket transport stream.
pub trait OkxWsStream:
    Sink<Req, Error = ProtocolError> + Stream<Item = Result<Resp, ProtocolError>>
{
}

impl<S> OkxWsStream for S
where
    S: Sink<Req, Error = ProtocolError>,
    S: Stream<Item = Result<Resp, ProtocolError>>,
{
}

type BoxStream = Pin<Box<dyn OkxWsStream + Send>>;

pin_project! {
    /// Okx websocket transport of v5 api.
    pub struct Transport {
        #[pin]
        inner: BoxStream,
        stream_id: usize,
    }
}

impl Transport {
    pub(crate) fn new<S, Err>(
        transport: S,
        ping_timeout: Duration,
        waker: Arc<AtomicWaker>,
    ) -> Transport
    where
        S: 'static + Send,
        Err: 'static,
        S: Sink<String, Error = Err>,
        S: Stream<Item = Result<String, Err>>,
        Err: Into<anyhow::Error>,
    {
        let transport = ping_pong::layer(transport, ping_timeout);
        let transport = message::layer(transport);
        let transport = frame::layer(transport);
        let transport = stream::layer(transport, waker);
        let inner = transport
            .sink_map_err(ProtocolError::from)
            .map_err(ProtocolError::from);
        Self {
            inner: Box::pin(inner),
            stream_id: 1,
        }
    }
}

impl Sink<Req> for Transport {
    type Error = ProtocolError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Req) -> Result<(), Self::Error> {
        self.project().inner.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl Stream for Transport {
    type Item = Result<Resp, ProtocolError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl TagStore<Req, Resp> for Transport {
    type Tag = usize;

    fn assign_tag(self: Pin<&mut Self>, r: &mut Req) -> Self::Tag {
        let this = self.project();
        let id = *this.stream_id;
        *this.stream_id += 1;
        r.id = id;
        id
    }

    fn finish_tag(self: Pin<&mut Self>, r: &Resp) -> Self::Tag {
        match r.as_ref() {
            Ok(s) => s.id,
            Err(e) => e.stream_id,
        }
    }
}

impl From<tokio_tower::Error<Transport, Req>> for ProtocolError {
    fn from(err: tokio_tower::Error<Transport, Req>) -> Self {
        Self::TokioTower(err.into())
    }
}

/// Okx websocket api protocol.
pub struct Protocol {
    waker: Arc<AtomicWaker>,
    inner: Client<Transport, ProtocolError, Req>,
}

impl Protocol {
    pub(crate) async fn init(
        websocket: WsStream,
        ping_timeout: Duration,
    ) -> Result<Self, ProtocolError> {
        let transport = websocket
            .with(|msg: String| async move { Ok(Message::Text(msg)) })
            .filter_map(|msg| async move {
                match msg {
                    Ok(msg) => match msg {
                        Message::Text(text) => Some(Ok(text)),
                        _ => None,
                    },
                    Err(err) => Some(Err(err)),
                }
            });
        let waker = Arc::new(AtomicWaker::default());
        let transport = Transport::new(transport, ping_timeout, waker.clone());
        Ok(Self {
            waker,
            inner: Client::with_error_handler(transport, |e| {
                tracing::error!("protocol error: {e}");
            }),
        })
    }
}

impl Service<Request> for Protocol {
    type Response = Response;
    type Error = ProtocolError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // wake up when the transport is dead.
        self.waker.register(cx.waker());
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let resp = self.inner.call(req.into_client_stream());
        async move {
            let resp = resp.await?;
            let resp = match resp {
                Ok(stream) => {
                    let mut stream = Box::pin(stream.peekable());
                    if let Some(frame) = stream.as_mut().peek().await {
                        trace!("wait header; peeked {frame:?}");
                        Response::Streaming(stream)
                    } else {
                        trace!("wait header; no header");
                        Response::Error(StatusKind::EmptyResponse)
                    }
                }
                Err(err) => Response::Error(err.kind),
            };
            Ok(resp)
        }
        .boxed()
    }
}
