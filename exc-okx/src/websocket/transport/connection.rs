use super::endpoint::Endpoint;
use super::protocol::Protocol;
use crate::error::OkxError;
use crate::websocket::types::{request::Request, response::Response};
use exc::transport::websocket::connector::WsConnector;
use futures::future::BoxFuture;
use futures::FutureExt;
use http::Uri;
use tower::timeout::TimeoutLayer;
use tower::{reconnect::Reconnect, util::BoxService};
use tower::{Service, ServiceBuilder, ServiceExt};

/// Create a connection to okx websocket api.
pub(crate) struct Connect {
    inner: WsConnector,
}

impl Connect {
    fn new(inner: WsConnector) -> Self {
        Self { inner }
    }
}

impl tower::Service<Uri> for Connect {
    type Response = BoxService<Request, Response, OkxError>;
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
            let svc = Protocol::init(conn)
                .await
                .map_err(|err| OkxError::Protocol(err.into()))?
                .map_err(|err| OkxError::Protocol(err.into()))
                .boxed();
            Ok(svc)
        }
        .boxed()
    }
}

/// Okx websocket connection.
pub(crate) struct Connection {
    inner: BoxService<Request, Response, OkxError>,
}

impl Connection {
    /// Create a new okx websocket connection.
    pub(crate) fn new(endpoint: &Endpoint) -> Self {
        let connector = ServiceBuilder::default()
            .option_layer(endpoint.connection_timeout.map(TimeoutLayer::new))
            .service(Connect::new(WsConnector::default()))
            .boxed();
        let conn = Reconnect::new::<<Connect as Service<Uri>>::Response, Uri>(
            connector,
            endpoint.uri.clone(),
        )
        .map_err(OkxError::Connection);
        Self {
            inner: BoxService::new(conn),
        }
    }
}

impl tower::Service<Request> for Connection {
    type Response = Response;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.inner.call(req)
    }
}
