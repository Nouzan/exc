use crate::websocket::types::callback::Callback;
use crate::websocket::types::frames::client::ClientFrame;
use crate::websocket::types::frames::server::ServerFrame;
use crate::websocket::types::request::ClientStream;
use crate::websocket::types::response::Status;
use crate::websocket::types::response::{ServerStream, StatusKind};
use futures::channel::mpsc::{self, SendError, UnboundedReceiver, UnboundedSender};
use futures::SinkExt;
use futures::{Sink, Stream, StreamExt};
use pin_project_lite::pin_project;
use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashSet};
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
enum StreamState {
    Idle,
    Open,
    LocalClosed,
    RemoteClosed,
    Closed,
}

struct StreamContext {
    sender: UnboundedSender<ServerFrame>,
    stream: Option<ServerStream>,
    state: StreamState,
    tag: Option<String>,
}

impl StreamContext {
    fn new(id: usize, cb: Callback) -> Self {
        let (server_frame_tx, server_frame_rx) = mpsc::unbounded();
        let stream = ServerStream {
            id,
            cb,
            inner: server_frame_rx.boxed(),
        };
        Self {
            sender: server_frame_tx,
            stream: Some(stream),
            state: StreamState::Idle,
            tag: None,
        }
    }
}

/// Stream layer errors.
#[derive(Debug, Error)]
pub enum StreamingError<E> {
    /// Transport error.
    #[error(transparent)]
    Transport(#[from] E),

    /// Sender error.
    #[error(transparent)]
    Sender(SendError),

    /// Idle stream missing.
    #[error("idle stream missing")]
    IdleStreamMissing,

    /// Borken streaming layer.
    #[error("broken streaming layer")]
    BlokenStreamingLayer,
}

pub(super) fn layer<T, E>(
    transport: T,
) -> impl Sink<ClientStream, Error = StreamingError<E>>
       + Stream<Item = Result<Result<ServerStream, Status>, StreamingError<E>>>
where
    E: Send + 'static + std::fmt::Display,
    T: Send + 'static,
    T: Sink<ClientFrame, Error = E>,
    T: Stream<Item = Result<ServerFrame, E>>,
{
    let (mut tx, mut rx) = transport.split();
    let (client_frame_tx, mut client_frame_rx) = mpsc::unbounded::<ClientFrame>();
    let (sender, mut client_stream_rx) = mpsc::unbounded::<ClientStream>();
    let (mut server_stream_tx, receiver) = mpsc::unbounded();
    let mut streams: BTreeMap<usize, StreamContext> = BTreeMap::default();
    let mut last_server_stream_tx = server_stream_tx.clone();
    let mut tags = HashSet::<String, RandomState>::new();
    let worker = async move {
        loop {
            tokio::select! {
                Some(mut client_stream) = client_stream_rx.next() => {
                    let cb = client_stream.cb.take().expect("client stream must contains a callback");
                    let id = client_stream.id;
                    let ctx = StreamContext::new(id, cb);
                    streams.insert(id, ctx);
                    let mut client_frame_tx = client_frame_tx.clone();
                    tokio::spawn(async move {
                        while let Some(mut frame) = client_stream.inner.next().await {
                            frame.stream_id = id;
                            if let Err(err) = client_frame_tx.send(frame).await {
                                error!("streaming client worker; send error id={id} err={err}");
                                break;
                            }
                        }
                    });
                }
                Some(client_frame) = client_frame_rx.next() => {
                    let id = client_frame.stream_id;
                    if let Some(ctx) = streams.get_mut(&id) {
                        let is_end_stream = client_frame.is_end_stream();
                        match ctx.state {
                            StreamState::Idle => {
                                if is_end_stream {
                                    ctx.state = StreamState::Closed;
                                    server_stream_tx.send(Ok(Err(Status { stream_id: id, kind: StatusKind::CloseIdleStream }))).await.map_err(StreamingError::Sender)?;
                                    streams.remove(&id);
                    trace!("stream {id}; idle -> closed");
                                    // client frame is ignored
                                    continue;
                                } else {
                                    // the first client frame is considered to be a stream header, so we need to check the tag.
                                    if let Some(tag) = client_frame.tag() {
                                        if tags.contains(&tag) {
                                            server_stream_tx.send(Ok(Err(Status { stream_id: id, kind: StatusKind::AlreadySubscribed(tag) }))).await.map_err(StreamingError::Sender)?;
                                            ctx.state = StreamState::Closed;
                                            streams.remove(&id);
                                            // client frame is ignored
                                            continue;
                                        } else {
                                            tags.insert(tag.clone());
                                            ctx.tag = Some(tag);
                                        }
                                    }
                                    ctx.state = StreamState::Open;
                    trace!("stream {id}; idle -> open");
                                    if let Some(stream) = ctx.stream.take() {
                                        server_stream_tx.send(Ok(Ok(stream))).await.map_err(StreamingError::Sender)?;
                                    } else {
                                        return Err(StreamingError::IdleStreamMissing);
                                    }
                                }
                            },
                            StreamState::Open => {
                                if is_end_stream {
                                    ctx.state = StreamState::LocalClosed;
                    trace!("stream {id}; open -> local-closed");
                                }
                            },
                            StreamState::RemoteClosed => {
                                if is_end_stream {
                                    ctx.state = StreamState::Closed;
                                    if let Some(tag) = ctx.tag.take() {
                                        tags.remove(&tag);
                                    }
                                    streams.remove(&id);
                                    debug!("stream {id} closed abnormally (remote -> local)");
                    trace!("stream {id}; remote-closed -> closed");
                                }
                            }
                            StreamState::LocalClosed | StreamState::Closed => {
                                warn!("streamming worker; trying to send a client frame from a closed or local closed stream: id={id}, ignored");
                                continue;
                            }
                        }
                    } else {
                        warn!("streaming worker; recevied an outdated client frame: {client_frame:?}, ignored");
                        continue;
                    }
                    tx.send(client_frame).await?;
                }
                Some(server_frame) = rx.next() => {
                    let frame = server_frame?;
                    let id = frame.stream_id;
                    let is_end_stream = frame.is_end_stream();
                    if let Some(ctx) = streams.get_mut(&id) {
                        match ctx.state {
                            StreamState::Idle => {
                                warn!("streaming worker; recevied a server frame from an idle stream: id={id}, ignored");
                            },
                            StreamState::Open => {
                                if is_end_stream {
                                    ctx.state = StreamState::RemoteClosed;
                                    warn!("streaming worker; received a remote close frame: id={id}");
                    trace!("stream {id}; open -> remote-closed");
                                }
                                let _ = ctx.sender.send(frame).await;
                            },
                            StreamState::LocalClosed => {
                                if is_end_stream {
                                    ctx.state = StreamState::Closed;
                                    let _ = ctx.sender.send(frame).await;
                                    if let Some(tag) = ctx.tag.take() {
                                        tags.remove(&tag);
                                    }
                                    debug!("stream {id} closed normally (local -> remote)");
                    trace!("stream {id}; local-closed -> closed");
                                    streams.remove(&id);
                                } else {
                                    let _ = ctx.sender.send(frame).await;
                                }
                            },
                            StreamState::RemoteClosed | StreamState::Closed => {
                                warn!("streaming worker; recevied a server frame from a closed or remote closed stream: id={id}, ignored");
                            }
                        }
                    } else {
                        warn!("streaming worker; received an outdated server frame: {frame:?}, ignored");
                    }
                }
                else => {
                    break;
                }
            }
        }
        Result::<(), _>::Err(StreamingError::BlokenStreamingLayer)
    };
    tokio::spawn(async move {
        if let Err(err) = worker.await {
            error!("streaming worker: {err}");
            let _ = last_server_stream_tx.send(Err(err)).await;
        }
    });
    Streaming { sender, receiver }
}

pin_project! {
    struct Streaming<E> {
        #[pin]
        sender: UnboundedSender<ClientStream>,
        #[pin]
        receiver: UnboundedReceiver<Result<Result<ServerStream, Status>, StreamingError<E>>>,
    }
}

impl<E> Sink<ClientStream> for Streaming<E> {
    type Error = StreamingError<E>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .sender
            .poll_ready(cx)
            .map_err(|err| StreamingError::Sender(err))
    }

    fn start_send(self: Pin<&mut Self>, item: ClientStream) -> Result<(), Self::Error> {
        self.project()
            .sender
            .start_send(item)
            .map_err(|err| StreamingError::Sender(err))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .sender
            .poll_flush(cx)
            .map_err(|err| StreamingError::Sender(err))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .sender
            .poll_close(cx)
            .map_err(|err| StreamingError::Sender(err))
    }
}

impl<E> Stream for Streaming<E> {
    type Item = Result<Result<ServerStream, Status>, StreamingError<E>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().receiver.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.receiver.size_hint()
    }
}
