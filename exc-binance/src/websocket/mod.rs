/// Binance websocket API errors.
pub mod error;

/// Binance websocket protocol.
pub mod protocol;

/// Binance websocket request.
pub mod request;

/// Binance websocket resposne.
pub mod response;

/// Binance websocket endpoint.
pub mod endpoint;

use std::task::{Context, Poll};

use futures::future::BoxFuture;
use tower::{util::BoxService, Service};

use self::{endpoint::WsEndpoint, error::WsError, request::WsRequest, response::WsResponse};

/// Binance websocket api service.
pub struct BinanceWebsocketApi {
    svc: BoxService<WsRequest, WsResponse, WsError>,
}

impl BinanceWebsocketApi {
    /// Endpoint of USD-M Futures API.
    pub fn usd_margin_futures() -> WsEndpoint {
        WsEndpoint::from_static("wss://fstream.binance.com/ws/bnbusdt@markPrice")
    }
}

impl Service<WsRequest> for BinanceWebsocketApi {
    type Response = WsResponse;
    type Error = WsError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: WsRequest) -> Self::Future {
        self.svc.call(req)
    }
}