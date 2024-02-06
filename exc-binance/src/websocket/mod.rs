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

pub(crate) mod connect;

use std::task::{Context, Poll};

use futures::future::BoxFuture;
use tower::{util::BoxService, Service};

use self::{
    connect::BinanceWsHost, endpoint::WsEndpoint, error::WsError, protocol::frame::Name,
    request::WsRequest, response::WsResponse,
};

/// Binance websocket api service.
pub struct BinanceWebsocketApi {
    svc: BoxService<WsRequest, WsResponse, WsError>,
}

impl BinanceWebsocketApi {
    /// Endpoint of USD-M Futures API.
    pub fn usd_margin_futures() -> WsEndpoint {
        WsEndpoint::new(
            BinanceWsHost::UsdMarginFutures,
            Name::new("markPrice").inst("bnbusdt"),
        )
    }

    /// Endpoint of Spot API.
    pub fn spot() -> WsEndpoint {
        WsEndpoint::new(BinanceWsHost::Spot, Name::new("miniTicker").inst("btcusdt"))
    }

    /// Endpoint of European Options API.
    pub fn european_options() -> WsEndpoint {
        WsEndpoint::new(
            BinanceWsHost::EuropeanOptions,
            Name::new("index").inst("BTC-USDT"),
        )
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
