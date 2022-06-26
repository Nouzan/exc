use std::time::Duration;

use super::protocol::{
    frame::{Name, RequestFrame},
    stream::MultiplexRequest,
};
use async_stream::stream;

/// Binance websocket request.
pub struct WsRequest {
    pub(crate) stream: bool,
    pub(crate) inner: MultiplexRequest,
}

impl WsRequest {
    /// Subscribe to a stream.
    pub fn subscribe(stream: Name) -> Self {
        Self {
            stream: true,
            inner: MultiplexRequest::new(|token| {
                stream! {
                    yield RequestFrame::subscribe(0, stream.clone());
                    let _ = token.await;
                    yield RequestFrame::unsubscribe(0, stream);
                }
            }),
        }
    }

    /// Set stream timeout. Default to the `default_stream_timeout` in protocol config.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.inner = self.inner.timeout(duration);
        self
    }
}

impl From<WsRequest> for MultiplexRequest {
    fn from(req: WsRequest) -> Self {
        req.inner
    }
}
