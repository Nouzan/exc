use std::{
    pin::Pin,
    sync::Arc,
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

use crate::websocket::error::WsError;

use super::{
    frame::{ClientFrame, RequestFrame},
    Shared,
};

/// Multiplex request.
pub struct MultiplexRequest {
    id: usize,
    stream: BoxStream<'static, RequestFrame>,
}

/// Multiplex response.
pub struct MultiplexResponse {}

/// Stream protocol layer.
pub(crate) fn layer<T>(
    transport: T,
) -> (
    impl Sink<MultiplexRequest, Error = WsError> + Stream<Item = Result<MultiplexResponse, WsError>>,
    Arc<Shared>,
)
where
    T: Sink<RequestFrame, Error = WsError> + Send + 'static,
    T: Stream<Item = Result<ClientFrame, WsError>>,
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

struct StreamingContext<T> {
    transport: T,
    c2w_rx: UnboundedReceiver<MultiplexRequest>,
    w2c_tx: UnboundedSender<Result<MultiplexResponse, WsError>>,
    state: Arc<Shared>,
}

impl<T> StreamingContext<T>
where
    T: Sink<RequestFrame, Error = WsError>,
    T: Stream<Item = Result<ClientFrame, WsError>>,
{
    async fn into_worker(self) {
        let Self {
            transport,
            mut c2w_rx,
            mut w2c_tx,
            state,
        } = self;
        let (tx, mut rx) = transport.split();
        loop {
            tokio::select! {
                c2w = c2w_rx.next() => {

                },
                frame = rx.next() => {

                }
            }
        }
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
        T: Stream<Item = Result<ClientFrame, WsError>>,
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
