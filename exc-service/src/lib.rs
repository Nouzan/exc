#![deny(missing_docs)]

//! Define the [`Request`] and [`ExcService`] traits, and provide some useful helper traits.

use futures::{future::BoxFuture, FutureExt};
use std::marker::PhantomData;
use tower::{Layer, Service, ServiceExt};

/// Exchange Error.
pub mod error;

/// Layer.
pub mod layer;

/// Traits.
pub mod traits;

/// The adapt layer.
pub mod adapt;

#[cfg(feature = "retry")]
/// Retry utils.
pub mod retry;

pub use layer::ExcLayer;
pub use {
    adapt::Adaptor,
    traits::{BoxCloneExcService, BoxExcService, ExcService, ExcServiceExt, IntoExc, Request},
};

use self::adapt::{Adapt, AdaptLayer, AdaptService};
pub use self::error::ExchangeError;

/// The core service wrapper of this crate, which implements
/// [`ExcService<T>`] *if* the request type of the underlying
/// service implements [`Adaptor<T>`].
///
/// With the help of [`Exc`], we can use a single type to represent
/// all the services that an exchange can provide.
///
/// For example, let `Exchange` be an api endpoint implementation of a exchange,
/// which implements [`Service<R>`], where `R` is the request type of the api endpoint.
/// Then `Exc<Exchange, R>` will implement `Service<SubscribeTickers>` and
/// `Service<PlaceOrder>`, as long as `R` implements both `Adaptor<SubscribeTickers>`
/// and `Adaptor<PlaceOrder>`.
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
    /// Create from the given [`ExcService`].
    pub fn new(service: C) -> Self {
        Self {
            channel: service,
            _req: PhantomData,
        }
    }

    /// Make a request using the underlying channel directly.
    pub async fn request(&mut self, request: Req) -> Result<C::Response, C::Error> {
        ServiceExt::<Req>::oneshot(&mut self.channel, request).await
    }

    /// Apply rate-limit layer to the channel.
    #[cfg(feature = "limit")]
    pub fn into_rate_limited(
        self,
        num: u64,
        per: std::time::Duration,
    ) -> Exc<tower::limit::RateLimit<C>, Req> {
        use tower::limit::RateLimitLayer;
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
    pub fn into_adapted<R>(self) -> Exc<Adapt<C, Req, R>, R>
    where
        R: Request,
        C: AdaptService<Req, R>,
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
