use crate::error::OkxError;

use super::event::Change;
use super::Args;
use either::Either;
use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Okx streaming response.
#[derive(Debug)]
pub struct Streaming(mpsc::UnboundedReceiver<Change>);

/// Okx websocket response kind.
#[derive(Debug)]
pub enum WsResponseKind {
    /// Streaming.
    Streaming(Streaming),
    /// Unsubscribe.
    Unsubscribe,
    /// Unary.
    Unary,
}

/// Okx websocket response.
#[derive(Debug)]
pub struct WsResponse {
    status: Result<(), OkxError>,
    id: Either<String, Args>,
    kind: WsResponseKind,
}

impl WsResponse {
    /// Get status of the response.
    pub fn status(&self) -> Result<&(), &OkxError> {
        self.status.as_ref()
    }
}
