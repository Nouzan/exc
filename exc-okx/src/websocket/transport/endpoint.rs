use super::connection::Connection;
use crate::websocket::Client;
use http::Uri;

/// Okx websocket endpoint.
pub struct Endpoint {
    pub(crate) uri: Uri,
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            uri: Uri::from_static("wss://wsaws.okex.com:8443/ws/v5/public"),
        }
    }
}

impl Endpoint {
    /// Connect and create a okx websocket channel.
    pub fn connect(&self) -> Client {
        let svc = Connection::new(self);
        Client { svc }
    }
}
