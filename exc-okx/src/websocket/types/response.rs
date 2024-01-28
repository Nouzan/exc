use crate::error::OkxError;

use super::{callback::Callback, frames::server::ServerFrame};
use exc_core::{types::ticker::Ticker, ExchangeError};
use futures::{future::BoxFuture, stream::BoxStream, FutureExt, Stream, StreamExt, TryStreamExt};
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
    #[error("received no response")]
    EmptyResponse,

    /// Unexpected response type.
    #[error("unexpected response type")]
    UnexpectedResponseType,

    /// Other.
    #[error(transparent)]
    Other(anyhow::Error),
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
        pub(crate) inner: BoxStream<'static, Result<ServerFrame, OkxError>>,
    }
}

impl Stream for ServerStream {
    type Item = Result<ServerFrame, OkxError>;

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
    Streaming(BoxStream<'static, Result<ServerFrame, OkxError>>),
    /// Error.
    Error(StatusKind),
    /// Reconnected.
    Reconnected,
}

impl Response {
    /// Convert into a streaming result.
    #[deprecated(since = "0.7.2", note = "please use `into_stream` instead")]
    pub fn into_result(
        self,
    ) -> Result<BoxStream<'static, Result<ServerFrame, OkxError>>, StatusKind> {
        self.into_stream()
    }

    /// Convert into a stream of [`ServerFrame`].
    pub fn into_stream(
        self,
    ) -> Result<BoxStream<'static, Result<ServerFrame, OkxError>>, StatusKind> {
        match self {
            Self::Streaming(stream) => Ok(stream),
            Self::Error(status) => Err(status),
            Self::Reconnected => Err(StatusKind::Other(anyhow::anyhow!("invalid response kind"))),
        }
    }

    /// Convert into a unary response.
    pub fn into_unary(
        self,
    ) -> Result<BoxFuture<'static, Result<ServerFrame, OkxError>>, StatusKind> {
        let mut stream = self.into_stream()?;
        Ok(async move {
            match stream.next().await {
                Some(result) => result,
                None => Err(OkxError::Api(StatusKind::EmptyResponse)),
            }
        }
        .boxed())
    }
}

impl TryFrom<Response> for BoxStream<'static, Result<Ticker, ExchangeError>> {
    type Error = ExchangeError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        match value {
            Response::Streaming(stream) => {
                let stream = stream
                    .skip(1)
                    .flat_map(|frame| {
                        let res: Result<Vec<Result<Ticker, OkxError>>, OkxError> =
                            frame.and_then(|f| f.try_into());
                        match res {
                            Ok(tickers) => futures::stream::iter(tickers).left_stream(),
                            Err(err) => {
                                futures::stream::once(async move { Err(err) }).right_stream()
                            }
                        }
                    })
                    .map_err(ExchangeError::from)
                    .boxed();
                Ok(stream)
            }
            Response::Error(status) => Err(OkxError::Api(status).into()),
            Response::Reconnected => Err(ExchangeError::Other(anyhow::anyhow!(
                "invalid response kind"
            ))),
        }
    }
}
