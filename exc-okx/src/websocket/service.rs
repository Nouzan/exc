use super::{request::WsRequest, response::WsResponse};
use crate::error::OkxError;
use exc::transport::websocket::WsStream;
use futures::future::BoxFuture;

/// Okx websocket API service.
pub struct OkxWebsocketService {}

impl OkxWebsocketService {
    pub(crate) async fn init(ws: WsStream) -> Result<Self, OkxError> {
        todo!()
    }
}

impl tower::Service<WsRequest> for OkxWebsocketService {
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
