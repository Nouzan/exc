use crate::{
    http::request::{Payload, Rest, RestRequest},
    websocket::request::WsRequest,
};

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
}
