use super::super::messages::request::WsRequest;

/// Client Frame.
#[derive(Debug, Clone)]
pub struct ClientFrame {
    pub(crate) stream_id: usize,
    pub(crate) inner: WsRequest,
}
