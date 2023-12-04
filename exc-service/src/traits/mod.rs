use super::adapt::{Adapt, AdaptLayer, AdaptService};
use crate::{Exc, ExchangeError};
use futures::future::BoxFuture;
use tower::{
    util::{BoxCloneService, BoxService, MapErr},
    Layer, Service,
};

use std::{
    fmt,
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
};

/// Sendable [`ExcService`].
#[cfg(feature = "send")]
pub mod send;

/// Request and Response binding.
pub trait Request: Sized {
    /// Response type.
    type Response;
}

/// An alias of [`Service`] that requires the input type to be a [`Request`],
/// and the error type to be [`ExchangeError`].
///
/// Basically, [`Request`] is just indicating that the input type has a fixed response type.
pub trait ExcService<R>
where
    R: Request,
{
    /// The future response value.
    type Future: Future<Output = Result<R::Response, ExchangeError>>;

    /// See [`Service::poll_ready`] for more details.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// See [`Service::call`] for more details.
    fn call(&mut self, req: R) -> Self::Future;

    /// Convert to a [`Service`].
    fn as_service(&mut self) -> AsService<'_, Self, R>
    where
        Self: Sized,
    {
        AsService {
            inner: self,
            _req: PhantomData,
        }
    }
}

impl<S, R> ExcService<R> for S
where
    S: Service<R, Response = R::Response, Error = ExchangeError>,
    R: Request,
{
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        Service::<R>::poll_ready(self, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        Service::<R>::call(self, req)
    }
}

/// [`Service`] returns by [`ExcService::as_service`].
pub struct AsService<'a, S, R> {
    inner: &'a mut S,
    _req: PhantomData<R>,
}

impl<'a, S, R> fmt::Debug for AsService<'a, S, R>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsService")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'a, S, R> Service<R> for AsService<'a, S, R>
where
    R: Request,
    S: ExcService<R>,
{
    type Response = R::Response;

    type Error = ExchangeError;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ExcService::<R>::poll_ready(self.inner, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        ExcService::<R>::call(self.inner, req)
    }
}

/// [`Service`] returns by [`ExcServiceExt::into_service`].
pub struct IntoService<S, R> {
    inner: S,
    _req: PhantomData<R>,
}

impl<S, R> fmt::Debug for IntoService<S, R>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoService")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<S, R> Clone for IntoService<S, R>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _req: PhantomData,
        }
    }
}

impl<S, R> Copy for IntoService<S, R> where S: Copy {}

impl<S, R> Service<R> for IntoService<S, R>
where
    R: Request,
    S: ExcService<R>,
{
    type Response = R::Response;

    type Error = ExchangeError;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ExcService::<R>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        ExcService::<R>::call(&mut self.inner, req)
    }
}

/// Extension trait for [`ExcService`].
pub trait ExcServiceExt<R>: ExcService<R>
where
    R: Request,
{
    /// Convert into a [`Service`].
    fn into_service(self) -> IntoService<Self, R>
    where
        Self: Sized,
    {
        IntoService {
            inner: self,
            _req: PhantomData,
        }
    }

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
    fn rate_limited(
        self,
        num: u64,
        per: std::time::Duration,
    ) -> tower::limit::RateLimit<IntoService<Self, R>>
    where
        Self: Sized,
    {
        use tower::limit::RateLimitLayer;
        self.into_service().apply(&RateLimitLayer::new(num, per))
    }

    /// Apply a retry layer to the service.
    #[cfg(feature = "retry")]
    fn retry(
        self,
        max_duration: std::time::Duration,
    ) -> tower::retry::Retry<crate::retry::Always, IntoService<Self, R>>
    where
        R: Clone,
        Self: Sized + Clone,
    {
        use crate::retry::Always;
        use tower::retry::RetryLayer;

        self.into_service()
            .apply(&RetryLayer::new(Always::with_max_duration(max_duration)))
    }

    /// Create a boxed [`ExcService`].
    fn boxed(self) -> BoxExcService<R>
    where
        Self: Sized + Send + 'static,
        R: Send + 'static,
        Self::Future: Send + 'static,
    {
        BoxExcService {
            inner: BoxService::new(self.into_service()),
        }
    }

    /// Create a boxed [`ExcService`] with [`Clone`].
    fn boxed_clone(&self) -> BoxCloneExcService<R>
    where
        R: Send + 'static,
        Self: Sized + Clone + Send + 'static,
        Self::Future: Send + 'static,
    {
        BoxCloneExcService {
            inner: BoxCloneService::new(self.clone().into_service()),
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
