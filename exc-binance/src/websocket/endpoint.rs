use std::time::Duration;

use exc_core::transport::websocket::connector::WsConnector;
use futures::FutureExt;
use http::Uri;
use tower::{reconnect::Reconnect, ServiceExt};

use super::{error::WsError, protocol::WsClient, request::WsRequest, BinanceWebsocketApi};

const DEFAULT_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_STREAM_TIMEOUT: Duration = Duration::from_secs(30);

/// A builder of binance websocket api service.
#[derive(Debug, Clone)]
pub struct WsEndpoint {
    uri: Uri,
    keep_alive_timeout: Option<Duration>,
    default_stream_timeout: Option<Duration>,
}

impl WsEndpoint {
    /// Create a new binance websocket api endpoint.
    pub fn new(uri: Uri) -> Self {
        Self {
            uri,
            keep_alive_timeout: None,
            default_stream_timeout: None,
        }
    }

    /// Create from static uri.
    pub fn from_static(src: &'static str) -> Self {
        Self::new(Uri::from_static(src))
    }

    /// Set the keep-alive timeout.
    pub fn keep_alive_timeout(&mut self, duration: Duration) -> &mut Self {
        self.keep_alive_timeout = Some(duration);
        self
    }

    /// Set the default stream timeout for each request stream.
    pub fn default_stream_timeout(&mut self, duration: Duration) -> &mut Self {
        self.default_stream_timeout = Some(duration);
        self
    }

    /// Connect to binance websocket api.
    pub fn connect(&self) -> BinanceWebsocketApi {
        let keep_alive_timeout = self
            .keep_alive_timeout
            .unwrap_or(DEFAULT_KEEP_ALIVE_TIMEOUT);
        let default_stream_timeout = self
            .default_stream_timeout
            .unwrap_or(DEFAULT_STREAM_TIMEOUT);
        let connect = WsConnector::default().and_then(move |ws| {
            async move {WsClient::with_websocket(ws, keep_alive_timeout, default_stream_timeout) }.boxed()
        });
        let connection = Reconnect::new::<WsClient, WsRequest>(connect, self.uri.clone()).map_err(
            |err| match err.downcast::<WsError>() {
                Ok(err) => *err,
                Err(err) => WsError::UnknownConnection(err),
            },
        );
        BinanceWebsocketApi {
            svc: connection.boxed(),
        }
    }
}
