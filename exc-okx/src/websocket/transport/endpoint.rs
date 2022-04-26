use super::connection::Connection;
use crate::{error::OkxError, websocket::OkxWsClient};
use http::Uri;

/// Okx websocket endpoint.
pub struct WsEndpoint {
    pub(crate) uri: Uri,
}

impl Default for WsEndpoint {
    fn default() -> Self {
        Self {
            uri: Uri::from_static("wss://wsaws.okex.com:8443/ws/v5/public"),
        }
    }
}

impl WsEndpoint {
    /// Connect and create a okx websocket channel.
    pub async fn connect(&self) -> Result<OkxWsClient, OkxError> {
        let svc = Connection::new(self);
        Ok(OkxWsClient { svc })
    }
}
