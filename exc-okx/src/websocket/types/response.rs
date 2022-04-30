use crate::error::OkxError;

use super::{callback::Callback, frames::server::ServerFrame};
use exc::types::ticker::Ticker;
use futures::{stream::BoxStream, Stream, StreamExt};
use pin_project_lite::pin_project;
use thiserror::Error;

/// Response error status kind.
#[derive(Debug, Error)]
pub enum StatusKind {
    /// Already subscribed.
    #[error("already subscribed")]
    AlreadySubscribed(String),

    /// Close an idle stream.
    #[error("close an idle stream")]
    CloseIdleStream,

    /// Empty response.
    #[error("empty response")]
    EmptyResponse,
}

/// Responsee error status.
#[derive(Debug)]
pub struct Status {
    pub(crate) stream_id: usize,
    /// Status kind.
    pub kind: StatusKind,
}

pin_project! {
    /// Server stream.
    pub struct ServerStream {
        pub(crate) id: usize,
        pub(crate) cb: Callback,
        #[pin]
        pub(crate) inner: BoxStream<'static, ServerFrame>,
    }
}

impl Stream for ServerStream {
    type Item = ServerFrame;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Okx websocket api response.
pub enum Response {
    /// Streaming.
    Streaming(BoxStream<'static, ServerFrame>),
    /// Error.
    Error(StatusKind),
}

impl Response {
    /// Convert into a result.
    pub fn into_result(self) -> Result<BoxStream<'static, ServerFrame>, StatusKind> {
        match self {
            Self::Streaming(stream) => Ok(stream),
            Self::Error(status) => Err(status),
        }
    }
}

impl TryFrom<Response> for BoxStream<'static, Result<Ticker, OkxError>> {
    type Error = OkxError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        match value {
            Response::Streaming(stream) => {
                let stream = stream.skip(1).map(|frame| frame.try_into()).boxed();
                Ok(stream)
            }
            Response::Error(status) => Err(OkxError::Api(status)),
        }
    }
}
