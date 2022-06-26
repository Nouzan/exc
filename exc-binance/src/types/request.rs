use crate::{
    http::request::{Payload, RestRequest},
    websocket::request::WsRequest,
};

/// Binance request.
pub enum Request {
    /// Http request.
    Http(RestRequest<Payload>),
    /// Websocket request.
    Ws(WsRequest),
}
