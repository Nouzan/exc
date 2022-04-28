use crate::error::OkxError;
use crate::websocket::types::messages::Args;
use crate::websocket::types::messages::{
    event::{Event, ResponseKind},
    request::WsRequest,
    response::WsResponse,
};
use either::Either;
use exc::transport::{driven::Driven, websocket::WsStream};
use futures::{Sink, SinkExt, Stream, StreamExt};
use pin_project_lite::pin_project;
use std::collections::{hash_map::RandomState, HashMap};
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;
use tokio_tower::multiplex::{Client, TagStore};
use tokio_tungstenite::tungstenite::Message;
use tower::Service;

mod frame;
mod message;
mod ping_pong;

pub use message::MessageError;
pub use ping_pong::PingPongError;

/// Protocol Error.
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// Transport Errors.
    #[error("transport: {0}")]
    Transport(#[from] MessageError<PingPongError>),

    /// Subsribed.
    #[error("subscribed: {0}")]
    Subscribed(Args),
}

/// Okx websocket transport stream.
pub trait OkxWsStream:
    Sink<WsRequest, Error = OkxError> + Stream<Item = Result<WsResponse, OkxError>>
{
}

impl<S> OkxWsStream for S
where
    S: Sink<WsRequest, Error = OkxError>,
    S: Stream<Item = Result<WsResponse, OkxError>>,
{
}

type BoxStream = Pin<Box<dyn OkxWsStream + Send>>;

pin_project! {
/// Okx websocket transport of v5 api.
pub struct Transport {
    #[pin]
    inner: BoxStream,
}
}

impl Transport {
    pub(crate) fn new<S, Err>(transport: S) -> Transport
    where
        S: 'static + Send,
        Err: 'static,
        S: Sink<String, Error = Err>,
        S: Stream<Item = Result<String, Err>>,
        Err: Into<anyhow::Error>,
    {
        let mut streams = HashMap::<_, _, RandomState>::default();
        let transport = ping_pong::layer(transport);
        let transport = message::layer(transport);
        let inner = transport
            .sink_map_err(|err| OkxError::Protocol(ProtocolError::from(err).into()))
            .filter_map(move |msg: Result<Event, _>| {
                let resp = match msg {
                    Ok(event) => match event {
                        Event::Response(resp) => match resp {
                            ResponseKind::Login(_) => {
                                error!("login unimplemented");
                                None
                            }
                            ResponseKind::Subscribe { arg } => {
                                let resp = if streams.contains_key(&arg) {
                                    WsResponse::from_error(
                                        Either::Right(arg.clone()),
                                        OkxError::Protocol(ProtocolError::Subscribed(arg).into()),
                                    )
                                } else {
                                    let (resp, tx) = WsResponse::streaming(arg.clone());
                                    streams.insert(arg, tx);
                                    resp
                                };
                                Some(Ok(resp))
                            }
                            ResponseKind::Unsubscribe { arg } => {
                                streams.remove(&arg);
                                Some(Ok(WsResponse::unsubscribed(arg)))
                            }
                            ResponseKind::Error(msg) => Some(WsResponse::from_failure(msg)),
                        },
                        Event::Change(change) => {
                            trace!("received {change:?}");
                            if let Some(stream) = streams.get_mut(&change.arg) {
                                if let Err(err) = stream.send(change) {
                                    let args = err.0.arg;
                                    debug!("the listener of {args:?} is gone");
                                    streams.remove(&args);
                                }
                            }
                            None
                        }
                    },
                    Err(err) => {
                        match &err {
                            MessageError::Transport(
                                PingPongError::RemoteClosed
                                | PingPongError::PingTimeout
                                | PingPongError::Ping(_),
                            ) => {
                                streams.clear();
                            }
                            _ => {}
                        }
                        error!("transport error: {err}");
                        Some(Err(OkxError::Protocol(
                            ProtocolError::Transport(err).into(),
                        )))
                    }
                };
                futures::future::ready(resp)
            });
        let inner = Box::pin(Driven::new(inner));
        Self { inner }
    }
}

impl Sink<WsRequest> for Transport {
    type Error = OkxError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: WsRequest) -> Result<(), Self::Error> {
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
    type Item = Result<WsResponse, OkxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl TagStore<WsRequest, WsResponse> for Transport {
    type Tag = String;

    fn assign_tag(self: Pin<&mut Self>, r: &mut WsRequest) -> Self::Tag {
        r.to_string()
    }

    fn finish_tag(self: Pin<&mut Self>, r: &WsResponse) -> Self::Tag {
        r.tag()
    }
}

impl From<tokio_tower::Error<Transport, WsRequest>> for OkxError {
    fn from(err: tokio_tower::Error<Transport, WsRequest>) -> Self {
        Self::Protocol(err.into())
    }
}

/// Okx websocket api protocol.
pub struct Protocol {
    inner: Client<Transport, OkxError, WsRequest>,
}

impl Protocol {
    pub(crate) async fn init(websocket: WsStream) -> Result<Self, OkxError> {
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
        let transport = Transport::new(transport);
        Ok(Self {
            inner: Client::new(transport),
        })
    }
}

impl Service<WsRequest> for Protocol {
    type Response = WsResponse;
    type Error = OkxError;
    type Future = <Client<Transport, OkxError, WsRequest> as Service<WsRequest>>::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: WsRequest) -> Self::Future {
        self.inner.call(req)
    }
}
