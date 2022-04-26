use crate::{
    error::OkxError,
    websocket::{WsRequest, WsResponse},
};
use futures::future::{poll_fn, BoxFuture};
use tower::Service;

use super::transport::connection::Connection;

/// Okx websocket channel.
pub struct OkxWsClient {
    pub(crate) svc: Connection,
}

impl OkxWsClient {
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

impl tower::Service<WsRequest> for OkxWsClient {
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
