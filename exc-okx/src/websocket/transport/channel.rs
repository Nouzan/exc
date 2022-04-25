use futures::future::BoxFuture;

use crate::{
    error::OkxError,
    websocket::{request::WsRequest, response::WsResponse},
};

use super::connection::Connection;

/// Okx websocket endpoint.
pub struct WsEndpoint {
    pub(crate) uri: http::Uri,
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
