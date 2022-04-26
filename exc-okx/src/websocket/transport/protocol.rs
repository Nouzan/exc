use crate::error::OkxError;
use crate::websocket::{
    types::{
        event::{Event, ResponseKind},
        request::WsRequestMessage,
    },
    WsRequest, WsResponse,
};
use either::Either;
use exc::transport::websocket::WsStream;
use futures::{ready, Future, Sink, SinkExt, Stream, StreamExt};
use pin_project_lite::pin_project;
use std::collections::{hash_map::RandomState, HashMap};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{Instant, Sleep};
use tokio_tower::multiplex::{Client, TagStore};
use tokio_tungstenite::tungstenite::Message;
use tower::Service;

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

#[derive(Clone, Copy)]
enum PingState {
    Idle,
    Ping,
    SendPing,
    PingSent,
    WaitPong,
    PingFailed,
}

pin_project! {
    struct PingPong<Si, S> {
        #[pin]
        stream: S,
        #[pin]
        sink: Si,
        #[pin]
        message_deadline: Sleep,
        #[pin]
        ping_deadline: Sleep,
        state: PingState,
    }
}

impl<Si, S> PingPong<Si, S> {
    const MESSAGE_TIMEOUT: std::time::Duration = Duration::from_secs(20);
    const PING: &'static str = "ping";

    fn new(sink: Si, stream: S) -> Self {
        let next = Instant::now() + Self::MESSAGE_TIMEOUT;
        let message_deadline = tokio::time::sleep_until(next);
        let ping_deadline = tokio::time::sleep_until(next);
        Self {
            sink,
            stream,
            message_deadline,
            ping_deadline,
            state: PingState::Idle,
        }
    }
}

impl<T, Si, S> Sink<T> for PingPong<Si, S>
where
    Si: Sink<T>,
{
    type Error = Si::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.project().sink.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().sink.poll_close(cx)
    }
}

impl<Si, S, Err> Stream for PingPong<Si, S>
where
    OkxError: From<Err>,
    S: Stream<Item = Result<String, Err>>,
    Si: Sink<String, Error = Err>,
{
    type Item = Result<String, OkxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.stream.poll_next(cx) {
            Poll::Ready(s) => match s {
                Some(Ok(s)) => {
                    let next = Instant::now() + Self::MESSAGE_TIMEOUT;
                    this.message_deadline.reset(next);
                    *this.state = PingState::Idle;
                    match s.as_str() {
                        "pong" => return Poll::Pending,
                        _ => return Poll::Ready(Some(Ok(s))),
                    }
                }
                Some(Err(err)) => {
                    return Poll::Ready(Some(Err(OkxError::from(err))));
                }
                None => {
                    return Poll::Ready(None);
                }
            },
            Poll::Pending => {}
        };
        match this.state {
            PingState::Idle => {
                ready!(this.message_deadline.poll(cx));
                let next = Instant::now() + Self::MESSAGE_TIMEOUT;
                this.ping_deadline.reset(next);
                *this.state = PingState::Ping;
                Poll::Pending
            }
            PingState::Ping => {
                match this.sink.poll_ready(cx) {
                    Poll::Ready(_) => {
                        *this.state = PingState::SendPing;
                    }
                    Poll::Pending => {}
                }
                ready!(this.ping_deadline.poll(cx));
                *this.state = PingState::PingFailed;
                Poll::Ready(Some(Err(OkxError::PingTimeout)))
            }
            PingState::SendPing => {
                let res = this.sink.start_send(Self::PING.to_string());
                *this.state = PingState::PingSent;
                if let Err(err) = res {
                    let err = OkxError::from(err);
                    *this.state = PingState::PingFailed;
                    return Poll::Ready(Some(Err(OkxError::Ping(err.into()))));
                }
                ready!(this.ping_deadline.poll(cx));
                *this.state = PingState::PingFailed;
                Poll::Ready(Some(Err(OkxError::PingTimeout)))
            }
            PingState::PingSent => {
                match this.sink.poll_flush(cx) {
                    Poll::Ready(_) => {
                        *this.state = PingState::WaitPong;
                        return Poll::Pending;
                    }
                    Poll::Pending => {}
                }
                ready!(this.ping_deadline.poll(cx));
                *this.state = PingState::PingFailed;
                Poll::Ready(Some(Err(OkxError::PingTimeout)))
            }
            PingState::WaitPong => {
                ready!(this.ping_deadline.poll(cx));
                *this.state = PingState::PingFailed;
                Poll::Ready(Some(Err(OkxError::PingTimeout)))
            }
            PingState::PingFailed => {
                Poll::Ready(Some(Err(OkxError::Ping(anyhow::anyhow!("ping failed")))))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

pin_project! {
/// Okx websocket transport of v5 api.
pub struct Transport {
    #[pin]
    inner: BoxStream,
}
}

impl Transport {
    pub(crate) fn new<S, Err>(base: S) -> Transport
    where
        S: 'static + Send,
        Err: 'static,
        S: Sink<String, Error = Err>,
        S: Stream<Item = Result<String, Err>>,
        OkxError: From<Err>,
    {
        let mut streams = HashMap::<_, _, RandomState>::default();
        let (sink, stream) = base.split();
        let ping_pong = PingPong::new(sink, stream);
        let inner = ping_pong
            .sink_map_err(OkxError::from)
            .with(|req: WsRequest| async move {
                let req: WsRequestMessage = req.into();
                let req = serde_json::to_string(&req)?;
                Ok(req)
            })
            .filter_map(move |msg: Result<String, _>| {
                let resp = match msg {
                    Ok(msg) => match serde_json::from_str::<Event>(&msg) {
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
                                            OkxError::SubscribedOrUnsubscribing(arg),
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
                            error!("deserializing error: msg={msg} err={err}");
                            Some(Err(err.into()))
                        }
                    },
                    Err(err) => Some(Err(err)),
                };
                futures::future::ready(resp)
            });
        let inner = Box::pin(inner);
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
