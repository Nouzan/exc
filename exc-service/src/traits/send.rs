use tower::Service;

use crate::{ExcService, ExchangeError, Request};

use std::{
    fmt,
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
};

/// An alias of [`ExcService`] with [`Send`] and `'static` bounds.
pub trait SendExcService<R>: Send + 'static
where
    R: Request,
{
    /// The future response value.
    type Future: Future<Output = Result<R::Response, ExchangeError>> + Send + 'static;

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

    /// Convert to a [`Service`].
    fn into_service(self) -> IntoService<Self, R>
    where
        Self: Sized,
    {
        IntoService {
            inner: self,
            _req: PhantomData,
        }
    }
}

impl<S, R> SendExcService<R> for S
where
    S: ExcService<R> + Send + 'static,
    R: Request,
    S::Future: Send + 'static,
{
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        ExcService::<R>::poll_ready(self, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        ExcService::<R>::call(self, req)
    }
}

/// [`Service`] returns by [`SendExcService::as_service`].
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
    S: SendExcService<R>,
{
    type Response = R::Response;

    type Error = ExchangeError;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        SendExcService::<R>::poll_ready(self.inner, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        SendExcService::<R>::call(self.inner, req)
    }
}

/// [`Service`] returns by [`SendExcService::into_service`].
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
    S: SendExcService<R>,
{
    type Response = R::Response;

    type Error = ExchangeError;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        SendExcService::<R>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        SendExcService::<R>::call(&mut self.inner, req)
    }
}
