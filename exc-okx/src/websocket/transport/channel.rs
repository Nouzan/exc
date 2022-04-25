use crate::{
    error::OkxError,
    websocket::{WsRequest, WsResponse},
};
use futures::future::{poll_fn, BoxFuture};
use http::Uri;
use tower::Service;

use super::connection::Connection;

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
    pub async fn connect(&self) -> Result<WsChannel, OkxError> {
        let svc = Connection::new(self);
        Ok(WsChannel { svc })
    }
}

/// Okx websocket channel.
pub struct WsChannel {
    svc: Connection,
}

impl WsChannel {
    /// Check if channel is ready.
    pub(crate) async fn ready(&mut self) -> Result<(), OkxError> {
        poll_fn(|cx| self.poll_ready(cx)).await?;
        Ok(())
    }

    /// Send request.
    pub async fn send(
        &mut self,
        request: WsRequest,
    ) -> Result<<Self as Service<WsRequest>>::Future, OkxError> {
        self.ready().await?;
        let fut = self.call(request);
        Ok(fut)
    }
}

impl tower::Service<WsRequest> for WsChannel {
    type Response = WsResponse;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: WsRequest) -> Self::Future {
        self.svc.call(req)
    }
}
