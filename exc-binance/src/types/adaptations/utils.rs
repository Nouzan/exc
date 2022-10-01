use exc_core::{types::utils::Reconnect, Adaptor};

use crate::{websocket::request::WsRequest, Request};

impl Adaptor<Reconnect> for Request {
    fn from_request(_req: Reconnect) -> Result<Self, exc_core::ExchangeError> {
        Ok(Self::Ws(WsRequest::reconnect()))
    }

    fn into_response(
        _resp: Self::Response,
    ) -> Result<<Reconnect as exc_core::Request>::Response, exc_core::ExchangeError> {
        Ok(())
    }
}
