use super::channel::WsEndpoint;
use crate::error::OkxError;
use crate::websocket::{request::WsRequest, response::WsResponse, service::OkxWebsocketService};
use exc::transport::websocket::connector::WsConnector;
use futures::future::BoxFuture;
use futures::FutureExt;
use http::Uri;
use tower::ServiceExt;
use tower::{reconnect::Reconnect, util::BoxService};

/// Create a connection to OKX websocket api.
pub(crate) struct Connect {
    inner: WsConnector,
}

impl Connect {
    fn new(inner: WsConnector) -> Self {
        Self { inner }
    }
}

impl tower::Service<Uri> for Connect {
    type Response = OkxWebsocketService;
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
            let svc = OkxWebsocketService::init(conn).await?;
            Ok(svc)
        }
        .boxed()
    }
}

/// Okx websocket connection.
pub(crate) struct Connection {
    inner: BoxService<WsRequest, WsResponse, OkxError>,
}

impl Connection {
    /// Create a new okx websocket connection.
    pub(crate) fn new(endpoint: &WsEndpoint) -> Self {
        let connector = Connect::new(WsConnector::default());
        let conn = Reconnect::new::<OkxWebsocketService, Uri>(connector, endpoint.uri.clone())
            .map_err(OkxError::Connection);
        Self {
            inner: BoxService::new(conn),
        }
    }
}

impl tower::Service<WsRequest> for Connection {
    type Response = WsResponse;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: WsRequest) -> Self::Future {
        self.inner.call(req)
    }
}
