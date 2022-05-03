use crate::error::OkxError;
use crate::websocket::types::{request::Request, response::Response};
use exc::ExchangeError;
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use tower::{buffer::Buffer, util::BoxService, Service, ServiceExt};

/// Okx websocket channel.
#[derive(Clone)]
pub struct Channel {
    pub(crate) svc: Buffer<BoxService<Request, Response, OkxError>, Request>,
}

impl Channel {
    /// Send request.
    pub async fn request(
        &mut self,
        request: Request,
    ) -> Result<<Self as Service<Request>>::Future, ExchangeError> {
        self.ready().await?;
        let fut = self.call(request);
        Ok(fut)
    }
}

impl tower::Service<Request> for Channel {
    type Response = Response;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc
            .poll_ready(cx)
            .map_err(OkxError::Buffer)
            .map_err(|err| ExchangeError::Other(err.into()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.svc
            .call(req)
            .map_err(OkxError::Buffer)
            .map_err(|err| ExchangeError::Other(err.into()))
            .boxed()
    }
}
