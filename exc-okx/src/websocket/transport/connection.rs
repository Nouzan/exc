use std::time::Duration;

use super::endpoint::Endpoint;
use super::protocol::Protocol;
use crate::error::OkxError;
use crate::key::Key;
use crate::websocket::types::messages::event::ResponseKind;
use crate::websocket::types::response::StatusKind;
use crate::websocket::types::{request::Request, response::Response};
use exc_core::transport::websocket::connector::WsConnector;
use futures::future::BoxFuture;
use futures::FutureExt;
use http::Uri;
use tower::timeout::TimeoutLayer;
use tower::{reconnect::Reconnect, util::BoxService};
use tower::{Service, ServiceBuilder, ServiceExt};

/// Create a connection to okx websocket api.
pub(crate) struct Connect {
    inner: WsConnector,
    ping_timeout: Duration,
    key: Option<Key>,
}

impl Connect {
    fn new(inner: WsConnector, ping_timeout: Duration, key: Option<&Key>) -> Self {
        Self {
            inner,
            ping_timeout,
            key: key.cloned(),
        }
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
        let ping_timeout = self.ping_timeout;
        let key = self.key.clone();
        async move {
            let conn = conn.await?;
            let mut svc = Protocol::init(conn, ping_timeout)
                .await
                .map_err(|err| OkxError::Protocol(err.into()))?
                .map_err(|err| OkxError::Protocol(err.into()))
                .boxed();
            tracing::trace!("protocol initialized");
            if let Some(key) = key {
                svc.ready().await?;
                let resp = svc
                    .call(Request::login(key)?)
                    .await?
                    .into_unary()
                    .map_err(OkxError::Api)?
                    .await?
                    .into_response()
                    .ok_or(OkxError::Api(StatusKind::EmptyResponse))?;
                match resp {
                    ResponseKind::Login(_) => {
                        tracing::trace!("login; login success");
                    }
                    ResponseKind::Error(err) => {
                        tracing::error!("login error; {err}");
                        return Err(OkxError::LoginError(anyhow::anyhow!("{err}")));
                    }
                    resp => {
                        tracing::error!("login error; unexpected response: {resp:?}");
                        return Err(OkxError::LoginError(anyhow::anyhow!(
                            "unexpected response: {resp:?}"
                        )));
                    }
                }
            }
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
            .service(Connect::new(
                WsConnector::default(),
                endpoint.ping_timeout,
                endpoint.login.as_ref(),
            ))
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
