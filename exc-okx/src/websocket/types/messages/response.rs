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

    /// Get response tag.
    pub fn tag(&self) -> String {
        match &self.id {
            Either::Left(id) => id.clone(),
            Either::Right(args) => match &self.kind {
                WsResponseKind::Streaming(_) => {
                    format!("sub:{args}")
                }
                WsResponseKind::Unsubscribe => {
                    format!("unsub:{args}")
                }
                _ => {
                    format!("unary:{args}")
                }
            },
        }
    }

    /// Convert into a stream.
    pub fn into_stream(self) -> BoxStream<'static, Change> {
        match self.kind {
            WsResponseKind::Streaming(s) => UnboundedReceiverStream::new(s.0).boxed(),
            WsResponseKind::Unary | WsResponseKind::Unsubscribe => futures::stream::empty().boxed(),
        }
    }
}
