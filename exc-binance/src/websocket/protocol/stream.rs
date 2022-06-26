use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
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
use tokio::time::Instant;

use crate::websocket::{error::WsError, protocol::frame::Op};

use super::frame::{Name, RequestFrame, ResponseFrame, ServerFrame, StreamFrame};

pub(crate) type ResponseToken = tokio::sync::oneshot::Receiver<()>;
type RequestToken = tokio::sync::oneshot::Sender<()>;

/// Multiplex request.
pub struct MultiplexRequest {
    pub(crate) id: usize,
    token: RequestToken,
    timeout: Option<Duration>,
    pub(crate) stream: BoxStream<'static, RequestFrame>,
}

impl MultiplexRequest {
    pub(crate) fn new<S, F>(stream: F) -> Self
    where
        F: FnOnce(ResponseToken) -> S,
        S: Stream<Item = RequestFrame> + Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let stream = stream(rx).boxed();
        Self {
            id: 0,
            token: tx,
            timeout: None,
            stream,
        }
    }

    pub(crate) fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
}

/// Multiplex response.
#[derive(Debug)]
pub struct MultiplexResponse {
    pub(crate) id: usize,
    token: RequestToken,
    rx: tokio::sync::mpsc::UnboundedReceiver<Result<ServerFrame, WsError>>,
}

impl MultiplexResponse {
    pub(crate) fn into_stream(self) -> impl Stream<Item = Result<ServerFrame, WsError>> {
        tokio_stream::wrappers::UnboundedReceiverStream::new(self.rx)
            .scan(self.token, |_, item| futures::future::ready(Some(item)))
    }
}

/// Stream protocol layer.
pub(super) fn layer<T>(
    transport: T,
    default_stream_timeout: Duration,
) -> (
    impl Sink<MultiplexRequest, Error = WsError> + Stream<Item = Result<MultiplexResponse, WsError>>,
    Arc<Shared>,
)
where
    T: Sink<RequestFrame, Error = WsError> + Send + 'static,
    T: Stream<Item = Result<ServerFrame, WsError>>,
{
    let (streaming, worker, cancel) = Streaming::new(transport, default_stream_timeout);
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
    /// First correct client frame is sent.
    Open,
    /// Client stream is closed.
    LocalClosing(Instant),
    /// Received a close stream frame.
    RemoteClosed,
}

struct StreamState {
    id: usize,
    tx: tokio::sync::mpsc::UnboundedSender<Result<ServerFrame, WsError>>,
    state: State,
    topic: Option<Name>,
    timeout: Duration,
}

impl StreamState {
    fn close(&mut self) -> bool {
        tracing::trace!("stream {} is closing, state={:?}", self.id, self.state);
        match self.state {
            State::Idle => false,
            State::Open | State::RemoteClosed => {
                self.state = State::LocalClosing(Instant::now() + self.timeout);
                true
            }
            State::LocalClosing(_) => true,
        }
    }

    fn handle_client_frame(
        &mut self,
        frame: &RequestFrame,
        topics: &mut HashMap<Name, usize>,
    ) -> Result<bool, WsError> {
        tracing::trace!(
            "stream {}: handling client frame, state={:?}",
            self.id,
            self.state
        );
        match self.state {
            State::Idle => {
                if let Op::Subscribe = frame.method {
                    if let Some(name) = frame.params.get(0) {
                        if topics.contains_key(name) {
                            self.send_server_frame(Err(WsError::StreamSubscribed(name.clone())))?;
                            Ok(false)
                        } else {
                            self.topic = Some(name.clone());
                            self.state = State::Open;
                            topics.insert(name.clone(), self.id);
                            Ok(true)
                        }
                    } else {
                        Ok(false)
                    }
                } else {
                    self.send_server_frame(Err(WsError::EmptyStreamName))?;
                    Ok(false)
                }
            }
            State::Open => Ok(true),
            State::LocalClosing(_) => Ok(true),
            State::RemoteClosed => Ok(false),
        }
    }

    fn send_server_frame(&mut self, frame: Result<ServerFrame, WsError>) -> Result<(), WsError> {
        match self.tx.send(frame) {
            Ok(_) => Ok(()),
            Err(_) => {
                tracing::trace!("stream {}: found response stream is gone during sending server frame, state={:?}", self.id, self.state);
                Ok(())
            }
        }
    }

    fn handle_response_frame(&mut self, frame: ResponseFrame) -> Result<bool, WsError> {
        tracing::trace!(
            "received a resposne frame: frame={frame:?}, id={}, state={:?}",
            self.id,
            self.state
        );
        match self.state {
            State::Idle => Ok(true),
            State::Open => {
                let is_close_stream = frame.is_close_stream();
                self.send_server_frame(Ok(ServerFrame::Response(frame)))?;
                // TODO: not sure what to do for now.
                if is_close_stream {
                    self.state = State::RemoteClosed;
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            State::LocalClosing(_) => {
                self.send_server_frame(Ok(ServerFrame::Response(frame)))?;
                tracing::trace!("assume that this is a close stream frame: id={}", self.id);
                Ok(false)
            }
            State::RemoteClosed => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}"))),
        }
    }

    fn handle_stream_frame(&mut self, frame: StreamFrame) -> Result<bool, WsError> {
        tracing::trace!(
            "received a stream frame: frame={frame:?}, id={}, state={:?}",
            self.id,
            self.state
        );
        match self.state {
            State::Idle => Ok(true),
            State::Open | State::LocalClosing(_) => {
                self.send_server_frame(Ok(ServerFrame::Stream(frame)))?;
                Ok(true)
            }
            State::RemoteClosed => Err(WsError::UnexpectedFrame(anyhow::anyhow!("{frame:?}"))),
        }
    }
}

struct ContextShared {
    c_tx: tokio::sync::mpsc::UnboundedSender<RequestFrame>,
    streams: Mutex<(HashMap<usize, StreamState>, HashMap<Name, usize>)>,
    timeout: Duration,
    cancel: tokio::sync::mpsc::UnboundedSender<()>,
}

impl ContextShared {
    fn new(
        c_tx: tokio::sync::mpsc::UnboundedSender<RequestFrame>,
        timeout: Duration,
    ) -> (Self, tokio::sync::mpsc::UnboundedReceiver<()>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (
            Self {
                c_tx,
                streams: Mutex::new((HashMap::default(), HashMap::default())),
                timeout,
                cancel: tx,
            },
            rx,
        )
    }

    fn handle_client_frame(&self, frame: &RequestFrame) -> bool {
        let id = frame.id;
        let (streams, topics) = &mut (*self.streams.lock().unwrap());
        if let Some(stream) = streams.get_mut(&id) {
            match stream.handle_client_frame(frame, topics) {
                Ok(keep_stream) => {
                    if keep_stream {
                        true
                    } else {
                        if let Some(topic) = streams.remove(&id).and_then(|state| state.topic) {
                            topics.remove(&topic);
                        }
                        false
                    }
                }
                Err(err) => {
                    tracing::error!("stream protocol error: {err}");
                    false
                }
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
        if let Err(err) = responser
            .send(Ok(MultiplexResponse {
                rx,
                id,
                token: request.token,
            }))
            .await
        {
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
                        topic: None,
                        timeout: request.timeout.unwrap_or(self.timeout),
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
                if ctx.handle_client_frame(&client_frame) {
                    if let Err(_) = c_tx.send(client_frame) {
                        break;
                    }
                } else {
                    break;
                }
            }
            tracing::trace!("response stream worker of {id}; finished");
        };
        let ctx = self.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = ctx.cancel.closed() => {
                    tracing::trace!("response stream worker of {id}; cancelled");
                }
                _ = response_stream_worker => {}
            }
            {
                let (streams, topics) = &mut (*ctx.streams.lock().unwrap());
                if let Some(stream) = streams.get_mut(&id) {
                    if !stream.close() {
                        if let Some(topic) = streams.remove(&id).and_then(|s| s.topic) {
                            topics.remove(&topic);
                        }
                    }
                }
            }
        });
        true
    }

    async fn handle_server_frame(self: &Arc<Self>, frame: ServerFrame) -> bool {
        let ctx = self.clone();
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
        if let Some((id, res)) = res {
            match res {
                Ok(keep_stream) => {
                    if !keep_stream {
                        if let Some(topic) = streams.remove(&id).and_then(|state| state.topic) {
                            topics.remove(&topic);
                        }
                    }
                }
                Err(err) => {
                    tracing::error!("stream protocol error; {err}");
                    return false;
                }
            }
        }
        true
    }
}

struct StreamingContext<T> {
    transport: T,
    c2w_rx: UnboundedReceiver<MultiplexRequest>,
    w2c_tx: UnboundedSender<Result<MultiplexResponse, WsError>>,
    state: Arc<Shared>,
    timeout: Duration,
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
            timeout,
        } = self;
        let (tx, mut rx) = transport.split();
        let (c_tx, c_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ctx, mut cancel) = ContextShared::new(c_tx, timeout);
        let shared = Arc::new(ctx);
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
        let ctx = shared.clone();
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
        let ctx = shared;
        let zombie_worker = async move {
            loop {
                tokio::time::sleep(timeout).await;
                let streams = ctx.streams.lock().unwrap();
                let now = Instant::now();
                for (id, stream) in streams.0.iter() {
                    if let State::LocalClosing(deadline) = stream.state {
                        if deadline > now {
                            tracing::trace!("zombie worker; found zombie {id}");
                            break;
                        }
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
            _ = zombie_worker => {
                tracing::trace!("zombie worker finished");
            }
        }
        state.waker.wake();
        cancel.close();
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
    fn new<T>(
        transport: T,
        default_stream_timeout: Duration,
    ) -> (Self, impl Future<Output = ()>, oneshot::Receiver<()>)
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
            timeout: default_stream_timeout,
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
