use futures::{Stream, TryStreamExt};

use super::{
    error::WsError,
    protocol::{frame::StreamFrame, stream::MultiplexResponse},
};

/// Binance websocket response.
pub struct WsResponse {
    pub(crate) inner: MultiplexResponse,
}

impl WsResponse {
    /// Convert into a stream.
    pub async fn into_stream<T>(self) -> Result<impl Stream<Item = Result<T, WsError>>, WsError>
    where
        T: TryFrom<StreamFrame, Error = WsError>,
    {
        let mut stream = self.inner.into_stream();
        if let Some(header) = stream.try_next().await? {
            tracing::trace!("ws response: header={header:?}");
            Ok(stream.and_then(|frame| async move { T::try_from(frame.into_stream_frame()?) }))
        } else {
            Err(WsError::NoResponse)
        }
    }
}

impl From<MultiplexResponse> for WsResponse {
    fn from(inner: MultiplexResponse) -> Self {
        Self { inner }
    }
}
