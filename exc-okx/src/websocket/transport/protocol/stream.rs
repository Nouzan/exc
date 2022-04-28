use crate::websocket::types::frames::client::ClientFrame;
use crate::websocket::types::frames::server::ServerFrame;
use crate::websocket::types::request::ClientStream;
use crate::websocket::types::response::{ServerStream, StatusKind};
use crate::websocket::types::{messages::request::WsRequest, response::Status};
use futures::channel::mpsc::{self, SendError, UnboundedReceiver, UnboundedSender};
use futures::SinkExt;
use futures::{Sink, Stream, StreamExt};
use pin_project_lite::pin_project;
use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashSet};
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

/// Stream layer errors.
#[derive(Debug, Error)]
pub enum StreamingError<E> {
    /// Transport error.
    #[error(transparent)]
    Transport(#[from] E),

    /// Sender error.
    #[error(transparent)]
    Sender(SendError),
}

pub(super) fn layer<T, E>(
    transport: T,
) -> impl Sink<ClientStream, Error = StreamingError<E>>
       + Stream<Item = Result<Result<ServerStream, Status>, StreamingError<E>>>
where
    E: Send + 'static,
    T: Send + 'static,
    T: Sink<ClientFrame, Error = E>,
    T: Stream<Item = Result<ServerFrame, E>>,
{
    let (mut tx, mut rx) = transport.split();
    let (mut client_frame_tx, mut client_frame_rx) = mpsc::unbounded::<ClientFrame>();
    let (sender, mut client_stream_rx) = mpsc::unbounded::<ClientStream>();
    let (mut server_stream_tx, receiver) = mpsc::unbounded();
    let mut streams: BTreeMap<
        usize,
        (
            UnboundedSender<ServerFrame>,
            Option<ServerStream>,
            Option<crate::websocket::types::messages::Args>,
        ),
    > = BTreeMap::default();
    let mut subscriptions = HashSet::<_, RandomState>::default();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    let worker = async move {
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let mut dead = Vec::new();
                    for (id, stream) in streams.iter() {
                        if stream.0.is_closed() {
                            dead.push(*id);
                        }
                    }
                    for id in dead {
                        if let Some(stream) = streams.remove(&id) {
                            if let Some(args) = stream.2 {
                                if subscriptions.remove(&args) {
                                    if let Err(err) = client_frame_tx.send(ClientFrame { stream_id: id, inner: WsRequest::Unsubscribe(args) }).await {
                                        error!("streaming worker: error sending client frame: id={id} err={err}");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Some(mut client_stream) = client_stream_rx.next() => {
                    let mut client_frame_tx = client_frame_tx.clone();
                    tokio::spawn(async move {
                        let id = client_stream.id;
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
                    match &client_frame.inner {
                        WsRequest::Subscribe(args) => {
                            if !subscriptions.contains(args) {
                                let (server_frame_tx, server_frame_rx) = mpsc::unbounded();
                                let stream = ServerStream {
                                    id,
                                    inner: server_frame_rx.boxed(),
                                };
                                streams.insert(id, (server_frame_tx, Some(stream), Some(args.clone())));
                                subscriptions.insert(args.clone());
                            } else {
                                if let Err(err) = server_stream_tx.send(Ok(Err(Status {
                                    stream_id: id,
                                    kind: StatusKind::AlreadySubscribed(args.clone()),
                                }))).await {
                                    error!("streaming worker; error sending already subscribed error (sink): id={id} err={err}");
                                    break;
                                }
                            }
                        },
                        WsRequest::Unsubscribe(args) => {
                            streams.remove(&id);
                            subscriptions.remove(args);
                        }
                    }
                    match tx.send(client_frame).await {
                        Ok(_) => {

                        },
                        Err(err) => {
                            if let Err(err) = server_stream_tx.send(Err(err.into())).await {
                                error!("streaming worker; error sending transport error (sink): err={err}");
                            }
                            break;
                        }
                    }
                }
                Some(server_frame) = rx.next() => {
                    match server_frame {
                        Ok(frame) => {
                            let id = frame.stream_id;
                            if let Some(server_stream) = streams.get_mut(&id) {
                                if let Err(err) = server_stream.0.send(frame).await {
                                    error!("streaming worker; error sending server frame: id={id} err={err}");
                                    if let Some(server_stream) = streams.remove(&id) {
                                        if let Some(args) = server_stream.2 {
                                            if subscriptions.remove(&args) {
                                                if let Err(err) = client_frame_tx.send(ClientFrame { stream_id: id, inner: WsRequest::Unsubscribe(args) }).await {
                                                    error!("streaming worker: error sending client frame: id={id} err={err}");
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    if let Some(stream) = server_stream.1.take() {
                                        if let Err(err) = server_stream_tx.send(Ok(Ok(stream))).await {
                                            error!("streaming worker; error sending server stream (stream): id={id} err={err}");
                                            break;
                                        }
                                    }
                                }
                            }
                        },
                        Err(err) => {
                            if let Err(err) = server_stream_tx.send(Err(err.into())).await {
                                error!("streaming worker; error sending transport error (stream): err={err}");
                            }
                            break;
                        }
                    }
                }
                else => {
                    break;
                }
            }
        }
    };
    tokio::spawn(async move {
        worker.await;
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
