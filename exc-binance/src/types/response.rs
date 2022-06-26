use futures::{Stream, TryStreamExt};

use crate::{
    http::{
        error::RestError,
        response::{Data, RestResponse},
    },
    websocket::{error::WsError, protocol::frame::StreamFrame, response::WsResponse},
    Error,
};

/// Binance response.
#[derive(Debug)]
pub enum Response {
    /// Http resposne.
    Http(RestResponse<Data>),
    /// Websocket response.
    Ws(WsResponse),
}

impl Response {
    /// As a stream of the given type.
    pub fn as_stream<T>(self) -> Option<impl Stream<Item = Result<T, Error>>>
    where
        T: TryFrom<StreamFrame, Error = WsError>,
    {
        match self {
            Self::Http(_) => None,
            Self::Ws(resp) => resp.into_stream().map(|stream| stream.map_err(Error::from)),
        }
    }

    /// Convert to a response of the given type.
    pub fn into_response<T>(self) -> Result<T, Error>
    where
        T: TryFrom<Data, Error = RestError>,
    {
        match self {
            Self::Http(resp) => resp.into_response().map_err(Error::from),
            Self::Ws(_) => Err(Error::WrongResponseType),
        }
    }
}
