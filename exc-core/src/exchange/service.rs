use std::task::{Context, Poll};

use futures::{future::MapErr, Future, TryFutureExt};
use tower::Service;

use crate::{Adapt, ExchangeError, Request};

/// Exc Service,
/// an alias of [`Service`].
pub trait ExcService<R>
where
    R: Request,
{
    /// Error type.
    type Error: Into<ExchangeError>;
    /// Future type.
    type Future: Future<Output = Result<R::Response, Self::Error>>;

    /// Check if the service is ready to process requests.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Process a request.
    fn call(&mut self, req: R) -> Self::Future;

    /// Convert into a [`Service`].
    fn into_service(self) -> Exc<Self>
    where
        Self: Sized,
    {
        Exc { inner: self }
    }

    /// Convert into a [`Adapt`].
    fn adapt(self) -> Adapt<Self, R>
    where
        Self: Sized,
    {
        Adapt::new(self)
    }

    /// Convert into a [`Service`] by ref.
    fn as_service_mut(&mut self) -> ExcMut<'_, Self> {
        ExcMut { inner: self }
    }
}

impl<S, R> ExcService<R> for S
where
    S: Service<R, Response = R::Response>,
    S::Error: Into<ExchangeError>,
    R: Request,
{
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(self, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        Service::call(self, req)
    }
}

/// A wrapper of exchange service.
#[derive(Debug, Clone, Copy)]
pub struct Exc<S> {
    inner: S,
}

impl<S, R> Service<R> for Exc<S>
where
    R: Request,
    S: ExcService<R>,
{
    type Response = R::Response;
    type Error = ExchangeError;
    type Future = MapErr<S::Future, fn(S::Error) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, req: R) -> Self::Future {
        self.inner
            .call(req)
            .map_err(<S as ExcService<R>>::Error::into)
    }
}

/// A wrapper of exchange service.
#[derive(Debug)]
pub struct ExcMut<'a, S: ?Sized> {
    inner: &'a mut S,
}

impl<'a, S, R> Service<R> for ExcMut<'a, S>
where
    R: Request,
    S: ExcService<R>,
{
    type Response = R::Response;
    type Error = ExchangeError;
    type Future = MapErr<S::Future, fn(S::Error) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, req: R) -> Self::Future {
        self.inner
            .call(req)
            .map_err(<S as ExcService<R>>::Error::into)
    }
}
