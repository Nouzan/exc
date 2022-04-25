use futures::future::BoxFuture;

use crate::error::OkxError;

use super::{request::WsRequest, response::WsResponse};

/// Okx websocket API service.
pub struct OkxWebsocket {}

impl tower::Service<WsRequest> for OkxWebsocket {
    type Response = WsResponse;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: WsRequest) -> Self::Future {
        todo!()
    }
}
