use std::time::Duration;

use crate::{
    http::layer::OkxHttpApiLayer, key::OkxKey, websocket::Endpoint as WsEndpoint, OkxRequest,
};
use exc_core::{transport::http, Exc, ExchangeError};
use tower::ServiceBuilder;

use super::Okx;

const CAP: usize = 512;

/// OKX endpoint.
pub struct Endpoint {
    ws: WsEndpoint,
    http: OkxHttpApiLayer<'static, fn(&ExchangeError) -> bool>,
    buffer: usize,
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            ws: WsEndpoint::default(),
            http: OkxHttpApiLayer::default(),
            buffer: CAP,
        }
    }
}

impl Endpoint {
    /// Connect and convert into an exc service.
    pub fn connect_exc(&self) -> Exc<Okx, OkxRequest> {
        Exc::new(self.connect())
    }

    /// Connect.
    pub fn connect(&self) -> Okx {
        let ws = self.ws.connect();
        let http = ServiceBuilder::default()
            .layer(&self.http)
            .service(http::endpoint::Endpoint::default().connect_https());
        Okx::new(ws, http, self.buffer)
    }

    /// Set ping timeout for the websocket channel.
    pub fn ws_ping_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.ws.ping_timeout(timeout);
        self
    }

    /// Set connection timeout for the websocket channel.
    pub fn ws_connection_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.ws.connection_timeout(timeout);
        self
    }

    /// Set request timeout for the websocket channel.
    pub fn ws_request_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.ws.request_timeout(timeout);
        self
    }

    /// Set the buffer capacity.
    pub fn buffer(&mut self, capacity: usize) -> &mut Self {
        self.buffer = capacity;
        self
    }

    /// Private mode (enable trading).
    pub fn private(&mut self, key: OkxKey) -> &mut Self {
        self.ws.private(key.clone());
        self.http.private(key);
        self
    }

    /// Set whether to use the testing endpoint.
    pub fn testing(&mut self, enable: bool) -> &mut Self {
        self.ws.testing(enable);
        self.http.testing(enable);
        self
    }
}
