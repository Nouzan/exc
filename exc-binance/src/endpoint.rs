use std::time::Duration;

use exc_core::transport::http::endpoint::Endpoint as HttpEndpoint;
use tower::{buffer::Buffer, ready_cache::ReadyCache, util::Either, ServiceBuilder};

use crate::{
    http::{layer::BinanceRestApiLayer, request::RestEndpoint},
    service::{Binance, BinanceInner, HTTP_KEY, WS_KEY},
    types::key::BinanceKey,
    websocket::{endpoint::WsEndpoint, BinanceWebsocketApi},
};

/// Binance endpoint.
#[derive(Debug)]
pub struct Endpoint {
    pub(crate) key: Option<BinanceKey>,
    pub(crate) http: (RestEndpoint, HttpEndpoint),
    pub(crate) ws: WsEndpoint,
}

impl Endpoint {
    /// Usd-margin futures endpoint.
    pub fn usd_margin_futures() -> Self {
        Self {
            key: None,
            http: (RestEndpoint::UsdMarginFutures, HttpEndpoint::default()),
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

    /// Set key.
    pub fn key(&mut self, key: BinanceKey) -> &mut Self {
        self.key = Some(key);
        self
    }

    /// Connect to binance service.
    pub fn connect(&self) -> Binance {
        let mut layer = BinanceRestApiLayer::new(self.http.0);
        if let Some(key) = self.key.as_ref() {
            layer = layer.key(key.clone());
        }
        let http = ServiceBuilder::default()
            .layer(layer)
            .service(self.http.1.connect_https());
        let ws = self.ws.connect();
        let mut svcs = ReadyCache::default();
        svcs.push(HTTP_KEY, Either::A(http));
        svcs.push(WS_KEY, Either::B(ws));
        let inner = Buffer::new(BinanceInner { svcs }, 256);
        Binance { inner }
    }
}
