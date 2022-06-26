use std::fmt;

use futures::{stream::BoxStream, Stream, StreamExt, TryStreamExt};

use crate::websocket::protocol::frame::ServerFrame;

use super::{
    error::WsError,
    protocol::{frame::StreamFrame, stream::MultiplexResponse},
};

/// Binance websocket response.
pub enum WsResponse {
    /// Raw.
    Raw(MultiplexResponse),
    /// Stream.
    Stream(BoxStream<'static, Result<StreamFrame, WsError>>),
}

impl fmt::Debug for WsResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(resp) => write!(f, "Raw({resp:?})"),
            Self::Stream(_) => write!(f, "Stream(_)"),
        }
    }
}

impl WsResponse {
    /// As a stream of the given type.
    pub fn into_stream<T>(self) -> Option<impl Stream<Item = Result<T, WsError>>>
    where
        T: TryFrom<StreamFrame, Error = WsError>,
    {
        match self {
            Self::Raw(_) => None,
            Self::Stream(stream) => {
                Some(stream.and_then(|frame| async move { T::try_from(frame) }))
            }
        }
    }

    pub(crate) async fn stream(self) -> Result<Self, WsError> {
        match self {
            Self::Raw(resp) => {
                let mut stream = resp.into_stream();
                if let Some(header) = stream.try_next().await? {
                    tracing::trace!("ws response: header={header:?}");
                    Ok(Self::Stream(
                        stream
                            .filter_map(|frame| {
                                let res = match frame {
                                    Ok(ServerFrame::Stream(frame)) => Some(Ok(frame)),
                                    Ok(ServerFrame::Response(resp)) => {
                                        tracing::trace!("received a response frame: {resp:?}");
                                        None
                                    }
                                    Err(err) => Some(Err(err)),
                                };
                                futures::future::ready(res)
                            })
                            .boxed(),
                    ))
                } else {
                    Err(WsError::NoResponse)
                }
            }
            Self::Stream(stream) => Ok(Self::Stream(stream)),
        }
    }
}

impl From<MultiplexResponse> for WsResponse {
    fn from(inner: MultiplexResponse) -> Self {
        Self::Raw(inner)
    }
}
