use crate::{ExcService, ExchangeError, Request};

use std::{
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
};

use super::{AsService, IntoService};

/// An alias of [`ExcService`] with [`Send`] and `'static` bounds.
pub trait SendExcService<R>: Send
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
    S: ExcService<R> + Send,
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
