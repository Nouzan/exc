use crate::{
    http::request::{Payload, Rest, RestRequest},
    websocket::request::WsRequest,
};

use super::Name;

/// Binance request.
pub enum Request {
    /// Http request.
    Http(RestRequest<Payload>),
    /// Websocket request.
    Ws(WsRequest),
}

impl Request {
    /// Create a request from rest payload.
    pub fn with_rest_payload<T>(payload: T) -> Self
    where
        T: Rest,
    {
        Self::Http(RestRequest::with_payload(payload))
    }

    /// Create a request to subscribe to a ws stream.
    pub fn subscribe(stream: Name) -> Self {
        Self::Ws(WsRequest::subscribe(stream))
    }

    /// Main stream subcribe.
    pub fn subcribe_main(stream: Name) -> Self {
        Self::Ws(WsRequest::main_stream(stream))
    }
}
