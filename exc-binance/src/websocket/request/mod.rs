use std::time::Duration;

use super::protocol::{
    frame::{Name, RequestFrame},
    stream::MultiplexRequest,
};
use async_stream::stream;

pub(crate) enum RequestKind {
    Multiplex(MultiplexRequest),
    Reconnect,
}

impl RequestKind {
    fn timeout(self, duration: Duration) -> Self {
        match self {
            Self::Multiplex(req) => Self::Multiplex(req.timeout(duration)),
            Self::Reconnect => Self::Reconnect,
        }
    }
}

/// Binance websocket request.
pub struct WsRequest {
    pub(crate) stream: bool,
    pub(crate) inner: RequestKind,
}

impl WsRequest {
    /// Subscribe to a stream.
    pub fn subscribe(stream: Name) -> Self {
        Self {
            stream: true,
            inner: RequestKind::Multiplex(MultiplexRequest::new(|token| {
                stream! {
                    yield RequestFrame::subscribe(0, stream.clone());
                    let _ = token.await;
                    yield RequestFrame::unsubscribe(0, stream);
                }
            })),
        }
    }

    /// Set stream timeout. Default to the `default_stream_timeout` in protocol config.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.inner = self.inner.timeout(duration);
        self
    }

    /// Subscribe to a main stream topic.
    pub fn main_stream(stream: Name) -> Self {
        Self {
            stream: true,
            inner: RequestKind::Multiplex(MultiplexRequest::main_stream(stream)),
        }
    }

    /// Reconnect.
    pub fn reconnect() -> Self {
        Self {
            stream: false,
            inner: RequestKind::Reconnect,
        }
    }
}

// impl From<WsRequest> for MultiplexRequest {
//     fn from(req: WsRequest) -> Self {
//         req.inner
//     }
// }
