use super::adapt::{Adapt, AdaptLayer, AdaptService};
use crate::{Exc, ExchangeError};
use futures::future::BoxFuture;
use tower::{
    util::{BoxCloneService, BoxService, MapErr},
    Layer, Service,
};

/// Request and Response binding.
pub trait Request: Sized {
    /// Response type.
    type Response;
}

/// An alias of [`Service`] that requires the input type to be a [`Request`],
/// and the error type to be [`ExchangeError`].
///
/// Basically, [`Request`] is just indicating that the input type has a fixed response type.
pub trait ExcService<R>: Service<R, Response = R::Response, Error = ExchangeError>
where
    R: Request,
{
}

impl<S, R> ExcService<R> for S
where
    S: Service<R, Response = R::Response, Error = ExchangeError>,
    R: Request,
{
}

/// Extension trait for [`ExcService`].
pub trait ExcServiceExt<R>: ExcService<R>
where
    R: Request,
{
    /// Apply a layer of which the result service is still a [`ExcService`].
    fn apply<L, R2>(self, layer: &L) -> L::Service
    where
        Self: Sized,
        R2: Request,
        L: Layer<Self>,
        L::Service: ExcService<R2>,
    {
        layer.layer(self)
    }

    /// Adapt the request type to the given.
    fn adapt<R2>(self) -> Adapt<Self, R, R2>
    where
        Self: Sized,
        R2: Request,
        Self: AdaptService<R, R2>,
    {
        self.apply(&AdaptLayer::default())
    }

    /// Apply a rate-limit layer to the service.
    #[cfg(feature = "limit")]
    fn rate_limited(self, num: u64, per: std::time::Duration) -> tower::limit::RateLimit<Self>
    where
        Self: Sized,
    {
        use tower::limit::RateLimitLayer;
        self.apply(&RateLimitLayer::new(num, per))
    }

    /// Apply a retry layer to the service.
    #[cfg(feature = "retry")]
    fn retry(
        self,
        max_duration: std::time::Duration,
    ) -> tower::retry::Retry<crate::retry::Always, Self>
    where
        R: Clone,
        Self: Sized + Clone,
    {
        use crate::retry::Always;
        use tower::retry::RetryLayer;

        self.apply(&RetryLayer::new(Always::with_max_duration(max_duration)))
    }

    /// Create a boxed [`ExcService`].
    fn boxed(self) -> BoxExcService<R>
    where
        Self: Sized + Send + 'static,
        Self::Future: Send + 'static,
    {
        BoxExcService {
            inner: BoxService::new(self),
        }
    }

    /// Create a boxed [`ExcService`] with [`Clone`].
    fn boxed_clone(&self) -> BoxCloneExcService<R>
    where
        Self: Sized + Clone + Send + 'static,
        Self::Future: Send + 'static,
    {
        BoxCloneExcService {
            inner: BoxCloneService::new(self.clone()),
        }
    }
}

impl<S, R> ExcServiceExt<R> for S
where
    S: ExcService<R>,
    R: Request,
{
}

type MapErrFn<E> = fn(E) -> ExchangeError;

/// Service that can be converted into a [`Exc`].
pub trait IntoExc<R>: Service<R, Response = R::Response>
where
    Self::Error: Into<ExchangeError>,
    R: Request,
{
    /// Convert into a [`Exc`].
    fn into_exc(self) -> Exc<MapErr<Self, MapErrFn<Self::Error>>, R>
    where
        Self: Sized,
    {
        Exc::new(MapErr::new(self, Self::Error::into))
    }
}

impl<S, R> IntoExc<R> for S
where
    S: Service<R, Response = R::Response>,
    S::Error: Into<ExchangeError>,
    R: Request,
{
}

/// Boxed [`ExcService`].
#[derive(Debug)]
pub struct BoxExcService<R>
where
    R: Request,
{
    inner: BoxService<R, R::Response, ExchangeError>,
}

impl<R> Service<R> for BoxExcService<R>
where
    R: Request,
{
    type Response = R::Response;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::<R>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        Service::<R>::call(&mut self.inner, req)
    }
}

/// Boxed [`ExcService`] with [`Clone`].
#[derive(Debug)]
pub struct BoxCloneExcService<R>
where
    R: Request,
{
    inner: BoxCloneService<R, R::Response, ExchangeError>,
}

impl<R> Clone for BoxCloneExcService<R>
where
    R: Request,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<R> Service<R> for BoxCloneExcService<R>
where
    R: Request,
{
    type Response = R::Response;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::<R>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        Service::<R>::call(&mut self.inner, req)
    }
}
