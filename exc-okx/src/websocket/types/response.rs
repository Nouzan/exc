use futures::stream::BoxStream;
use thiserror::Error;

use super::{frames::server::ServerFrame, messages::Args};

/// Response error status kind.
#[derive(Debug, Error)]
pub enum StatusKind {
    /// Already subscribed.
    #[error("already subscribed")]
    AlreadySubscribed(Args),
}

/// Responsee error status.
#[derive(Debug)]
pub struct Status {
    pub(crate) stream_id: usize,
    /// Status kind.
    pub kind: StatusKind,
}

/// Server stream.
pub struct ServerStream {
    pub(crate) id: usize,
    pub(crate) inner: BoxStream<'static, ServerFrame>,
}

/// Okx websocket api response.
pub enum Response {
    /// Streaming.
    Streaming(BoxStream<'static, ServerFrame>),
    /// Error.
    Error(Status),
}

impl Response {
    /// Convert into a result.
    pub fn into_result(self) -> Result<BoxStream<'static, ServerFrame>, Status> {
        match self {
            Self::Streaming(stream) => Ok(stream),
            Self::Error(status) => Err(status),
        }
    }
}
