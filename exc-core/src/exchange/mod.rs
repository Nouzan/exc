use futures::{future::BoxFuture, FutureExt};
use std::{marker::PhantomData, time::Duration};
use tower::{limit::RateLimit, Service, ServiceBuilder, ServiceExt};

use crate::{
    types::{Adaptor, Request},
    ExchangeError,
};

/// Layer.
pub mod layer;

/// Service.
pub mod service;

pub use layer::ExchangeLayer;
pub use service::ExchangeService;

/// Exchange.
#[derive(Debug)]
pub struct Exchange<C, Req> {
    channel: C,
    _req: PhantomData<fn() -> Req>,
}

impl<C, Req> Clone for Exchange<C, Req>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.channel.clone())
    }
}

impl<C, Req> Exchange<C, Req> {
    /// Create a new exchange client from the given channel.
    pub fn new(channel: C) -> Self {
        Self {
            channel,
            _req: PhantomData,
        }
    }

    /// Make a request using the underlying channel directly.
    pub async fn request(&mut self, request: Req) -> Result<C::Response, C::Error>
    where
        C: Service<Req>,
    {
        ServiceExt::<Req>::oneshot(&mut self.channel, request).await
    }

    /// Convert into a rate-limited service.
    pub fn into_rate_limited(self, num: u64, per: Duration) -> RateLimit<Self> {
        ServiceBuilder::default().rate_limit(num, per).service(self)
    }
}

impl<C, Req, R> Service<R> for Exchange<C, Req>
where
    R: Request,
    R::Response: Send + 'static,
    Req: Adaptor<R>,
    C: ExchangeService<Req>,
    C::Error: Into<ExchangeError>,
    C::Future: Send + 'static,
{
    type Response = R::Response;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.channel.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, req: R) -> Self::Future {
        let request = Req::from_request(req);
        match request {
            Ok(req) => {
                let res = self.channel.call(req);
                async move {
                    let resp = res.await.map_err(|err| err.into())?;
                    let resp = Req::into_response(resp)?;
                    Ok(resp)
                }
                .left_future()
            }
            Err(err) => futures::future::ready(Err(err)).right_future(),
        }
        .boxed()
    }
}
