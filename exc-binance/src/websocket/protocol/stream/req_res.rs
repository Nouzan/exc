use std::time::Duration;

use futures::{
    future::ready,
    stream::{once, BoxStream},
    Stream, StreamExt,
};

use crate::websocket::{
    error::WsError,
    protocol::frame::{Name, RequestFrame, ServerFrame, StreamFrame},
};

use tokio::sync::broadcast;

pub(crate) type ResponseToken = tokio::sync::oneshot::Receiver<()>;
type RequestToken = tokio::sync::oneshot::Sender<()>;

pub(crate) enum MultiplexRequestKind {
    MainStream(Name),
    SubStream {
        token: RequestToken,
        timeout: Option<Duration>,
        stream: BoxStream<'static, RequestFrame>,
    },
}

/// Multiplex request.
pub struct MultiplexRequest {
    pub(crate) id: usize,
    pub(crate) kind: MultiplexRequestKind,
}

impl MultiplexRequest {
    pub(crate) fn main_stream(name: Name) -> Self {
        Self {
            id: 0,
            kind: MultiplexRequestKind::MainStream(name),
        }
    }

    pub(crate) fn new<S, F>(stream: F) -> Self
    where
        F: FnOnce(ResponseToken) -> S,
        S: Stream<Item = RequestFrame> + Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let stream = stream(rx).boxed();
        Self {
            id: 0,
            kind: MultiplexRequestKind::SubStream {
                token: tx,
                timeout: None,
                stream,
            },
        }
    }

    pub(crate) fn timeout(mut self, duration: Duration) -> Self {
        match &mut self.kind {
            MultiplexRequestKind::MainStream(_) => {}
            MultiplexRequestKind::SubStream { timeout, .. } => {
                *timeout = Some(duration);
            }
        }
        self
    }
}

/// Multiplex response.
#[derive(Debug)]
pub enum MultiplexResponse {
    /// Main stream.
    MainStream(usize, Option<broadcast::Receiver<StreamFrame>>),
    /// Sub stream.
    SubStream {
        /// Id.
        id: usize,
        /// Token.
        token: RequestToken,
        /// Rx.
        rx: tokio::sync::mpsc::UnboundedReceiver<Result<ServerFrame, WsError>>,
    },
}

impl MultiplexResponse {
    pub(crate) fn into_stream(self) -> impl Stream<Item = Result<ServerFrame, WsError>> {
        match self {
            Self::MainStream(_, rx) => match rx {
                Some(rx) => {
                    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(
                        |res| match res {
                            Ok(frame) => ready(Some(Ok(ServerFrame::Stream(frame)))),
                            Err(_) => ready(None),
                        },
                    );
                    once(ready(Ok(ServerFrame::Empty)))
                        .chain(stream)
                        .left_stream()
                }
                None => once(ready(Err(WsError::MainStreamNotFound))).right_stream(),
            }
            .left_stream(),
            Self::SubStream { token, rx, .. } => {
                tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
                    .scan(token, |_, item| futures::future::ready(Some(item)))
                    .right_stream()
            }
        }
    }
}
