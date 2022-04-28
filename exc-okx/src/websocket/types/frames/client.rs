use super::super::messages::request::WsRequest;
use super::Frame;

/// Client Frame.
#[derive(Debug, Clone)]
pub struct ClientFrame {
    pub(crate) stream_id: usize,
    pub(crate) inner: WsRequest,
}

impl Frame for ClientFrame {
    fn stream_id(&self) -> usize {
        self.stream_id
    }
}
