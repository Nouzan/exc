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
            inner: MultiplexRequest::new(|mut token| {
                stream! {
                    yield RequestFrame::subscribe(0, stream.clone());
                    let _ = token.recv().await;
                    yield RequestFrame::unsubscribe(0, stream);
                    let _ = token.recv().await;
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
