use futures::{future::BoxFuture, FutureExt};
use http::Uri;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Error, MaybeTlsStream, WebSocketStream};

/// Websocket Stream.
pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type BoxConnecting = BoxFuture<'static, Result<WsStream, Error>>;

/// Websocket Connector.
pub struct WsConnector {}

impl WsConnector {
    /// Create a new websocket connector.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WsConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl tower::Service<Uri> for WsConnector {
    type Response = WsStream;
    type Error = Error;
    type Future = BoxConnecting;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        async move {
            tracing::trace!("ws connecting {req}");
            let (conn, _) = connect_async(req).await?;
            tracing::trace!("ws connected");
            Ok(conn)
        }
        .boxed()
    }
}
