use std::{
    future::Future,
    task::{Context, Poll},
};

use crate::{Exchange, ExchangeError};

use super::types::Request;
use futures::{future::MapErr, TryFutureExt};
use tower::Service;

/// Subscribe tickers.
pub mod subscribe_tickers;

/// Trade.
pub mod trade;

/// Book.
pub mod book;

/// Subscribe instruments.
pub mod instrument;

/// Fetch candles.
pub mod fetch_candles;

/// Trading service.
pub mod trading;

/// Exchange Service,
/// an alias of [`tower::Service`].
pub trait ExchangeService<R>
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

    /// Convert into a [`Exchange`].
    fn into_exchange(self) -> Exchange<Self, R>
    where
        Self: Sized,
    {
        Exchange::new(self)
    }

    /// Convert into a [`Service`] by ref.
    fn as_service_mut(&mut self) -> ExcMut<'_, Self> {
        ExcMut { inner: self }
    }
}

impl<S, R> ExchangeService<R> for S
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
    S: ExchangeService<R>,
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
            .map_err(<S as ExchangeService<R>>::Error::into)
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
    S: ExchangeService<R>,
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
            .map_err(<S as ExchangeService<R>>::Error::into)
    }
}
