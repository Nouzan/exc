use std::time::Duration;

use exc::transport::http::endpoint::Endpoint as HttpEndpoint;
use tower::{buffer::Buffer, ready_cache::ReadyCache, util::Either, ServiceBuilder};

use crate::{
    http::layer::BinanceRestApiLayer,
    service::{Binance, BinanceInner, HTTP_KEY, WS_KEY},
    websocket::{endpoint::WsEndpoint, BinanceWebsocketApi},
};

/// Binance endpoint.
#[derive(Debug)]
pub struct Endpoint {
    pub(crate) http: HttpEndpoint,
    pub(crate) ws: WsEndpoint,
}

impl Endpoint {
    /// Usd-margin futures endpoint.
    pub fn usd_margin_futures() -> Self {
        Self {
            http: HttpEndpoint::default(),
            ws: BinanceWebsocketApi::usd_margin_futures(),
        }
    }

    /// Set websocket keep-alive timeout.
    pub fn ws_keep_alive_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.ws.keep_alive_timeout(timeout);
        self
    }

    /// Set websocket default stream timeout.
    pub fn ws_default_stream_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.ws.default_stream_timeout(timeout);
        self
    }

    /// Connect to binance service.
    pub fn connect(&self) -> Binance {
        let http = ServiceBuilder::default()
            .layer(BinanceRestApiLayer)
            .service(self.http.connect_https());
        let ws = self.ws.connect();
        let mut svcs = ReadyCache::default();
        svcs.push(HTTP_KEY, Either::A(http));
        svcs.push(WS_KEY, Either::B(ws));
        let inner = Buffer::new(BinanceInner { svcs }, 256);
        Binance { inner }
    }
}
