use std::time::Duration;

use exc_core::transport::http::endpoint::Endpoint as HttpEndpoint;
use tower::{buffer::Buffer, ready_cache::ReadyCache, util::Either, ServiceBuilder};

use crate::{
    http::{layer::BinanceRestApiLayer, request::RestEndpoint},
    service::{Binance, BinanceInner, HTTP_KEY, WS_KEY},
    types::key::BinanceKey,
    websocket::{endpoint::WsEndpoint, BinanceWebsocketApi},
};

const CAP: usize = 512;

/// Binance endpoint.
pub struct Endpoint {
    pub(crate) key: Option<BinanceKey>,
    pub(crate) http: (RestEndpoint, HttpEndpoint),
    pub(crate) ws: WsEndpoint,
    buffer: usize,
}

impl Endpoint {
    /// Usd-margin futures endpoint.
    pub fn usd_margin_futures() -> Self {
        Self {
            key: None,
            http: (RestEndpoint::UsdMarginFutures, HttpEndpoint::default()),
            ws: BinanceWebsocketApi::usd_margin_futures(),
            buffer: CAP,
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

    /// Set listen key refresh retrys.
    pub fn ws_listen_key_retry(&mut self, retry: usize) -> &mut Self {
        self.ws.listen_key_retry(retry);
        self
    }

    /// Set listen key refresh interval.
    pub fn ws_listen_key_refresh_interval(&mut self, interval: Duration) -> &mut Self {
        self.ws.listen_key_refresh_interval(interval);
        self
    }

    /// Private mode.
    pub fn private(&mut self, key: BinanceKey) -> &mut Self {
        self.key = Some(key);
        self
    }

    /// Set buffer capacity.
    pub fn buffer(&mut self, capacity: usize) -> &mut Self {
        self.buffer = capacity;
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
        let ws = if self.key.is_some() {
            let private = http.clone();
            self.ws.clone().private(private).connect()
        } else {
            self.ws.connect()
        };
        let mut svcs = ReadyCache::default();
        svcs.push(HTTP_KEY, Either::A(http));
        svcs.push(WS_KEY, Either::B(ws));
        let inner = Buffer::new(BinanceInner { svcs }, self.buffer);
        Binance { inner }
    }
}
