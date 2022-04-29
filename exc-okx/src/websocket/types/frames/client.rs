use super::super::messages::request::WsRequest;

/// Client Frame.
#[derive(Debug, Clone)]
pub struct ClientFrame {
    pub(crate) stream_id: usize,
    pub(crate) inner: WsRequest,
}

impl ClientFrame {
    pub(crate) fn is_end_stream(&self) -> bool {
        matches!(self.inner, WsRequest::Unsubscribe(_))
    }

    pub(crate) fn tag(&self) -> Option<String> {
        match &self.inner {
            WsRequest::Subscribe(args) => Some(args.to_string()),
            _ => None,
        }
    }
}
