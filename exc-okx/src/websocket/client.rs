use crate::error::OkxError;
use futures::future::BoxFuture;
use tower::{Service, ServiceExt};

use super::{
    transport::connection::Connection,
    types::{request::Request, response::Response},
};

/// Okx websocket channel.
pub struct OkxWsClient {
    pub(crate) svc: Connection,
}

impl OkxWsClient {
    /// Send request.
    pub async fn send(
        &mut self,
        request: Request,
    ) -> Result<<Self as Service<Request>>::Future, OkxError> {
        self.ready().await?;
        let fut = self.call(request);
        Ok(fut)
    }
}

impl tower::Service<Request> for OkxWsClient {
    type Response = Response;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.svc.call(req)
    }
}
