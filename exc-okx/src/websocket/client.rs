use super::transport::channel::Channel;
use super::types::{request::Request, response::Response};
use crate::error::OkxError;
use exc::types::subscriptions::SubscribeTickers;
use exc::types::ticker::Ticker;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::FutureExt;
use tower::ServiceExt;

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
    pub async fn request(&mut self, request: Request) -> Result<Response, OkxError> {
        ServiceExt::<Request>::oneshot(self, request).await
    }

    /// Subscribe tickers.
    pub async fn subscribe_tickers(
        &mut self,
        inst: &str,
    ) -> Result<BoxStream<'static, Result<Ticker, OkxError>>, OkxError> {
        ServiceExt::<SubscribeTickers>::oneshot(self, SubscribeTickers::new(inst)).await
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

impl tower::Service<SubscribeTickers> for Client {
    type Response = BoxStream<'static, Result<Ticker, OkxError>>;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.channel.poll_ready(cx)
    }

    fn call(&mut self, req: SubscribeTickers) -> Self::Future {
        let request = Request::try_from(req);
        match request {
            Ok(req) => {
                let res = self.call(req);
                async move { res.await?.try_into() }.left_future()
            }
            Err(err) => futures::future::ready(Err(err)).right_future(),
        }
        .boxed()
    }
}
