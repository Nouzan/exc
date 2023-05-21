use futures::{future::BoxFuture, FutureExt};
use std::{
    marker::PhantomData,
    task::{Context, Poll},
    time::Duration,
};
use tower::{
    limit::{RateLimit, RateLimitLayer},
    Layer, Service, ServiceExt,
};

use crate::ExchangeError;

/// Layer.
pub mod layer;

/// Traits.
pub mod traits;

/// Adapt.
pub mod adapt;

pub use layer::ExcLayer;
pub use traits::{Adaptor, ExcService, IntoExc, Request};

use self::adapt::{AdaptLayer, Adapted};

/// A wrapper that convert a general purpose exchange service
/// into the specific exc services that are supported.
#[derive(Debug)]
pub struct Exc<C, Req> {
    channel: C,
    _req: PhantomData<fn() -> Req>,
}

impl<C, Req> Clone for Exc<C, Req>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
            _req: PhantomData,
        }
    }
}

impl<C, Req> Exc<C, Req> {
    /// Into the inner channel.
    #[inline]
    pub fn into_inner(self) -> C {
        self.channel
    }
}

impl<C, Req> Exc<C, Req>
where
    Req: Request,
    C: ExcService<Req>,
{
    /// Create a new exchange client from the given channel.
    pub fn new(channel: C) -> Self {
        Self {
            channel,
            _req: PhantomData,
        }
    }

    /// Make a request using the underlying channel directly.
    pub async fn request(&mut self, request: Req) -> Result<C::Response, C::Error> {
        ServiceExt::<Req>::oneshot(&mut self.channel, request).await
    }

    /// Apply rate-limit layer to the channel.
    pub fn into_rate_limited(self, num: u64, per: Duration) -> Exc<RateLimit<C>, Req> {
        self.into_layered(&RateLimitLayer::new(num, per))
    }

    #[cfg(feature = "retry")]
    /// Apply retry layer to the channel.
    pub fn into_retry(
        self,
        max_duration: std::time::Duration,
    ) -> Exc<tower::retry::Retry<crate::retry::Always, C>, Req>
    where
        Req: Clone,
        C: Clone,
    {
        use crate::retry::Always;
        use tower::retry::RetryLayer;

        self.into_layered(&RetryLayer::new(Always::with_max_duration(max_duration)))
    }

    /// Adapt the request type of the underlying channel to the target type `R`.
    pub fn into_adapted<R>(self) -> Exc<Adapted<C, Req, R>, R>
    where
        R: Request,
        R::Response: Send + 'static,
        Req: Adaptor<R>,
        C::Future: Send + 'static,
    {
        self.into_layered(&AdaptLayer::default())
    }

    /// Apply a layer to the underlying channel.
    pub fn into_layered<T, R>(self, layer: &T) -> Exc<T::Service, R>
    where
        T: Layer<C>,
        R: Request,
        T::Service: ExcService<R>,
    {
        Exc {
            channel: layer.layer(self.channel),
            _req: PhantomData,
        }
    }
}

impl<C, Req, R> Service<R> for Exc<C, Req>
where
    R: Request,
    R::Response: Send + 'static,
    Req: Adaptor<R>,
    C: ExcService<Req>,
    C::Future: Send + 'static,
{
    type Response = R::Response;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.channel.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let request = Req::from_request(req);
        match request {
            Ok(req) => {
                let res = self.channel.call(req);
                async move {
                    let resp = res.await?;
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

/// A wrapper of exchange service.
#[derive(Debug)]
pub struct ExcMut<'a, S: ?Sized> {
    pub(crate) inner: &'a mut S,
}

impl<'a, S, R> Service<R> for ExcMut<'a, S>
where
    R: Request,
    S: ExcService<R>,
{
    type Response = R::Response;
    type Error = ExchangeError;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        self.inner.call(req)
    }
}
