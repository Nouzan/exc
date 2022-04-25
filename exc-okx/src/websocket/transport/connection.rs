use super::channel::WsEndpoint;
use crate::error::OkxError;
use crate::websocket::{request::WsRequest, response::WsResponse, service::OkxWebsocket};
use exc::transport::websocket::connector::WsConnector;
use futures::future::BoxFuture;
use futures::FutureExt;
use http::Uri;
use tower::ServiceExt;
use tower::{reconnect::Reconnect, util::BoxService};

/// Create a connection to OKX websocket api.
pub struct Connect {
    inner: WsConnector,
}

impl Connect {
    fn new(inner: WsConnector) -> Self {
        Self { inner }
    }
}

impl tower::Service<Uri> for Connect {
    type Response = OkxWebsocket;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(OkxError::from)
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        let conn = self.inner.call(req);
        async move {
            let conn = conn.await?;
            todo!()
        }
        .boxed()
    }
}

/// Okx websocket connection.
pub struct Connection {
    inner: BoxService<WsRequest, WsResponse, OkxError>,
}

impl Connection {
    /// Create a new okx websocket connection.
    pub fn new(endpoint: WsEndpoint) -> Self {
        let connector = Connect::new(WsConnector::default());
        let conn = Reconnect::new::<OkxWebsocket, Uri>(connector, endpoint.uri)
            .map_err(OkxError::Connection);
        Self {
            inner: BoxService::new(conn),
        }
    }
}
