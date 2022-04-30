use super::transport::channel::Channel;
use super::types::{request::Request, response::Response};
use crate::error::OkxError;
use futures::future::BoxFuture;
use tower::{Service, ServiceExt};

/// Okx websocket client.
#[derive(Clone)]
pub struct Client {
    pub(crate) channel: Channel,
}

impl Client {
    /// Create a new client from the given channel.
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }

    /// Make a general request.
    pub async fn request(
        &mut self,
        request: Request,
    ) -> Result<<Self as Service<Request>>::Future, OkxError> {
        self.ready().await?;
        let fut = self.call(request);
        Ok(fut)
    }
}

impl tower::Service<Request> for Client {
    type Response = Response;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.channel.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.channel.call(req)
    }
}
