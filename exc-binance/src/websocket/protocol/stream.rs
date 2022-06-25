use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use futures::{
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    stream::BoxStream,
    task::AtomicWaker,
    Future, Sink, SinkExt, Stream, StreamExt,
};

use crate::websocket::{error::WsError, protocol::frame::Op};

use super::frame::{Name, RequestFrame, ResponseFrame, ServerFrame, StreamFrame};

pub(crate) type ResponseToken = tokio::sync::mpsc::UnboundedReceiver<()>;
type RequestToken = tokio::sync::mpsc::UnboundedSender<()>;

/// Multiplex request.
pub struct MultiplexRequest {
    pub(crate) id: usize,
    token: RequestToken,
    pub(crate) stream: BoxStream<'static, RequestFrame>,
}

impl MultiplexRequest {
    pub(crate) fn new<S, F>(stream: F) -> Self
    where
        F: FnOnce(ResponseToken) -> S,
        S: Stream<Item = RequestFrame> + Send + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let stream = stream(rx).boxed();
        Self {
            id: 0,
            token: tx,
            stream,
        }
    }
}

/// Multiplex response.
pub struct MultiplexResponse {
    pub(crate) id: usize,
    rx: tokio::sync::mpsc::UnboundedReceiver<Result<ServerFrame, WsError>>,
}

impl MultiplexResponse {
    pub(crate) fn into_stream(self) -> impl Stream<Item = Result<ServerFrame, WsError>> {
        tokio_stream::wrappers::UnboundedReceiverStream::new(self.rx)
    }
}

/// Stream protocol layer.
pub(super) fn layer<T>(
    transport: T,
) -> (
    impl Sink<MultiplexRequest, Error = WsError> + Stream<Item = Result<MultiplexResponse, WsError>>,
    Arc<Shared>,
)
where
    T: Sink<RequestFrame, Error = WsError> + Send + 'static,
    T: Stream<Item = Result<ServerFrame, WsError>>,
{
    let (streaming, worker, cancel) = Streaming::new(transport);
    tokio::spawn(async move {
        tokio::select! {
            _ = worker => {
                tracing::trace!("stream protocol worker finished");
            },
            _ = cancel => {
                tracing::trace!("stream protocol cancelled");
            }
        }
    });
    let state = streaming.state.clone();
    (streaming, state)
}

#[derive(Default)]
pub(super) struct Shared {
    waker: AtomicWaker,
}

impl Shared {
    pub(super) fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), WsError>> {
        self.waker.register(cx.waker());
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Idle,
    Open,
    LocalClosed,
    RemoteClosed,
}

struct StreamState {
    id: usize,
    tx: tokio::sync::mpsc::UnboundedSender<Result<ServerFrame, WsError>>,
    token: RequestToken,
    state: State,
    topic: Option<Name>,
}

impl StreamState {
    fn handle_client_frame(
        &mut self,
        frame: &RequestFrame,
        topics: &mut HashMap<Name, usize>,
    ) -> bool {
        match self.state {
            State::Idle => {
                if let Op::Subscribe = frame.method {
                    if let Some(name) = frame.params.get(0) {
                        if topics.contains_key(name) {
                            false
                        } else {
                            self.topic = Some(name.clone());
                            self.state = State::Open;
                            topics.insert(name.clone(), self.id);
                            true
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            State::Open => {
                if let Op::Unsubscribe = frame.method {
                    self.state = State::LocalClosed;
                    true
                } else {
                    false
                }
            }
            State::LocalClosed => false,
            State::RemoteClosed => false,
        }
    }

    fn handle_response_frame(&mut self, frame: ResponseFrame) -> bool {
        tracing::trace!(
            "received a resposne frame: frame={frame:?}, id={}, state={:?}",
            self.id,
            self.state
        );
        match self.state {
            State::Idle => true,
            State::Open => {
                // TODO: not sure what to do for now.
                if frame.is_close_stream() {
                    self.state = State::RemoteClosed;
                    false
                } else {
                    true
                }
            }
            State::LocalClosed => {
                tracing::trace!("assume it is a close stream frame: id={}", self.id);
                false
            }
            State::RemoteClosed => false,
        }
    }

    fn handle_stream_frame(&mut self, frame: StreamFrame) -> bool {
        tracing::trace!(
            "received a stream frame: frame={frame:?}, id={}, state={:?}",
            self.id,
            self.state
        );
        match self.state {
            State::Idle => true,
            State::Open | State::LocalClosed => {
                // TODO: send a close stream frame when error in open state.
                match self.tx.send(Ok(ServerFrame::Stream(frame))) {
                    Ok(_) => true,
                    Err(_) => {
                        let _ = self.token.send(());
                        false
                    }
                }
            }
            State::RemoteClosed => false,
        }
    }
}

struct ContextShared {
    c_tx: tokio::sync::mpsc::UnboundedSender<RequestFrame>,
    streams: Mutex<(HashMap<usize, StreamState>, HashMap<Name, usize>)>,
}

impl ContextShared {
    fn new(c_tx: tokio::sync::mpsc::UnboundedSender<RequestFrame>) -> Self {
        Self {
            c_tx,
            streams: Mutex::new((HashMap::default(), HashMap::default())),
        }
    }

    async fn handle_client_frame(&self, frame: &RequestFrame) -> bool {
        let id = frame.id;
        let (streams, topics) = &mut (*self.streams.lock().unwrap());
        if let Some(stream) = streams.get_mut(&id) {
            if stream.handle_client_frame(frame, topics) {
                true
            } else {
                if let Some(topic) = streams.remove(&id).and_then(|state| state.topic) {
                    topics.remove(&topic);
                }
                false
            }
        } else {
            false
        }
    }

    async fn handle_request(
        self: &Arc<Self>,
        responser: &mut UnboundedSender<Result<MultiplexResponse, WsError>>,
        mut request: MultiplexRequest,
    ) -> bool {
        let id = request.id;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        if let Err(err) = responser.send(Ok(MultiplexResponse { rx, id })).await {
            tracing::trace!("stream worker; failed to send response: err={err}");
            return false;
        }
        {
            let mut streams = self.streams.lock().unwrap();
            if streams.0.contains_key(&id) {
                let _ = tx.send(Err(WsError::DuplicateStreamId));
            } else {
                streams.0.insert(
                    id,
                    StreamState {
                        id: request.id,
                        tx: tx.clone(),
                        state: State::Idle,
                        token: request.token,
                        topic: None,
                    },
                );
            }
        }
        let ctx = self.clone();
        let response_stream_worker = async move {
            let id = request.id;
            tracing::trace!("response stream worker; start: id={id}");
            let c_tx = ctx.c_tx.clone();
            while let Some(mut client_frame) = request.stream.next().await {
                client_frame.id = request.id;
                if ctx.handle_client_frame(&client_frame).await {
                    if let Err(_) = c_tx.send(client_frame) {
                        break;
                    }
                } else {
                    break;
                }
            }
            tracing::trace!("response stream worker; finished: id={id}");
        };
        let ctx = self.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = response_stream_worker => {},
                _ = tx.closed() => {

                }
            }
            tracing::trace!("response stream worker; cancel stream: id={id}");
            {
                let (streams, topics) = &mut (*ctx.streams.lock().unwrap());
                if let Some(topic) = streams.remove(&id).and_then(|state| state.topic) {
                    topics.remove(&topic);
                }
            }
        });
        true
    }

    async fn handle_server_frame(self: &Arc<Self>, frame: ServerFrame) -> bool {
        let ctx = self.clone();
        tokio::spawn(async move {
            let (streams, topics) = &mut (*ctx.streams.lock().unwrap());
            let res = match frame {
                ServerFrame::Response(frame) => {
                    let id = frame.id;
                    streams.get_mut(&id).map(|stream| {
                        let good = stream.handle_response_frame(frame);
                        (id, good)
                    })
                }
                ServerFrame::Stream(frame) => frame
                    .to_name()
                    .and_then(|name| topics.get(&name))
                    .and_then(|id| streams.get_mut(&id))
                    .map(|stream| {
                        let good = stream.handle_stream_frame(frame);
                        let id = stream.id;
                        (id, good)
                    }),
            };
            if let Some((id, good)) = res {
                if !good {
                    if let Some(topic) = streams.remove(&id).and_then(|state| state.topic) {
                        topics.remove(&topic);
                    }
                }
            }
        });
        true
    }
}

struct StreamingContext<T> {
    transport: T,
    c2w_rx: UnboundedReceiver<MultiplexRequest>,
    w2c_tx: UnboundedSender<Result<MultiplexResponse, WsError>>,
    state: Arc<Shared>,
}

impl<T> StreamingContext<T>
where
    T: Sink<RequestFrame, Error = WsError>,
    T: Stream<Item = Result<ServerFrame, WsError>>,
{
    async fn into_worker(self) {
        let Self {
            transport,
            mut c2w_rx,
            mut w2c_tx,
            state,
        } = self;
        let (tx, mut rx) = transport.split();
        let (c_tx, c_rx) = tokio::sync::mpsc::unbounded_channel();
        let ctx = Arc::new(ContextShared::new(c_tx));
        let c_rx = tokio_stream::wrappers::UnboundedReceiverStream::new(c_rx).map(Ok);
        let sink_worker = async move {
            match c_rx.forward(tx).await {
                Ok(_) => {
                    tracing::debug!("`c_rx` finished");
                }
                Err(err) => {
                    tracing::error!("sink worker error: {err}");
                }
            }
        };
        let stream_worker = async move {
            loop {
                tokio::select! {
                    Some(c2w) = c2w_rx.next() => {
                        if !ctx.handle_request(&mut w2c_tx, c2w).await {
                            break;
                        }
                    },
                    Some(server_frame) = rx.next() => {
                        match server_frame {
                            Ok(server_frame) => {
                                if !ctx.handle_server_frame(server_frame).await {
                                    break;
                                }
                            },
                            Err(err) => {
                                tracing::error!("stream worker; server stream error: {err}");
                                break;
                            }
                        }

                    },
                    else => {
                        tracing::trace!("stream worker; one of the streams end");
                        break;
                    }
                }
            }
        };
        tokio::select! {
            _ = sink_worker => {
                tracing::trace!("sink worker finished");
            },
            _ = stream_worker => {
                tracing::trace!("stream worker finished");
            }
        }
        state.waker.wake();
    }
}

pin_project_lite::pin_project! {
    /// Streaming transport.
    struct Streaming {
        #[pin]
        c2w_tx: UnboundedSender<MultiplexRequest>,
        #[pin]
        w2c_rx: UnboundedReceiver<Result<MultiplexResponse, WsError>>,
        state: Arc<Shared>,
        _cancel: oneshot::Sender<()>,
    }
}

impl Streaming {
    fn new<T>(transport: T) -> (Self, impl Future<Output = ()>, oneshot::Receiver<()>)
    where
        T: Sink<RequestFrame, Error = WsError>,
        T: Stream<Item = Result<ServerFrame, WsError>>,
    {
        let (c2w_tx, c2w_rx) = mpsc::unbounded();
        let (w2c_tx, w2c_rx) = mpsc::unbounded();
        let (_cancel, cancel) = oneshot::channel();
        let state = Arc::new(Shared::default());
        let streaming = Self {
            c2w_tx,
            w2c_rx,
            state: state.clone(),
            _cancel,
        };
        let ctx = StreamingContext {
            transport,
            w2c_tx,
            c2w_rx,
            state,
        };
        (streaming, ctx.into_worker(), cancel)
    }
}

impl Sink<MultiplexRequest> for Streaming {
    type Error = WsError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.c2w_tx.poll_ready(cx).map_err(|_| {
            this.state.waker.wake();
            WsError::TransportIsBoken
        })
    }

    fn start_send(self: Pin<&mut Self>, item: MultiplexRequest) -> Result<(), Self::Error> {
        let this = self.project();
        if let Err(_) = this.c2w_tx.start_send(item) {
            this.state.waker.wake();
            return Err(WsError::TransportIsBoken);
        }
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.c2w_tx.poll_flush(cx).map_err(|_| {
            this.state.waker.wake();
            WsError::TransportIsBoken
        })
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.c2w_tx.poll_close(cx).map_err(|_| {
            this.state.waker.wake();
            WsError::TransportIsBoken
        })
    }
}

impl Stream for Streaming {
    type Item = Result<MultiplexResponse, WsError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().w2c_rx.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.w2c_rx.size_hint()
    }
}
