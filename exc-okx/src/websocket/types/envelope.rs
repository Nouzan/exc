use crate::{
    error::OkxError,
    websocket::{WsRequest, WsResponse},
};
use tokio::sync::oneshot;

/// Envelope for request.
#[derive(Debug)]
pub(crate) struct Envelope {
    pub(crate) request: WsRequest,
    pub(crate) callback: oneshot::Sender<Result<WsResponse, OkxError>>,
}

impl Envelope {
    pub(crate) fn new(
        request: WsRequest,
    ) -> (Self, oneshot::Receiver<Result<WsResponse, OkxError>>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                request,
                callback: tx,
            },
            rx,
        )
    }
}
