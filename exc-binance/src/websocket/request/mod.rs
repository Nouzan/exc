use super::protocol::{
    frame::{Name, RequestFrame},
    stream::MultiplexRequest,
};
use async_stream::stream;

/// Binance websocket request.
pub struct WsRequest {
    pub(crate) inner: MultiplexRequest,
}

impl WsRequest {
    /// Subscribe to a stream.
    pub fn subscribe(stream: Name) -> Self {
        Self {
            inner: MultiplexRequest::new(|token| {
                stream! {
                    yield RequestFrame::subscribe(0, stream.clone());
                    let _ = token.await;
                    yield RequestFrame::unsubscribe(0, stream);
                }
            }),
        }
    }
}

impl From<WsRequest> for MultiplexRequest {
    fn from(req: WsRequest) -> Self {
        req.inner
    }
}
