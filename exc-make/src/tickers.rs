use exc_service::{ExcService, ExchangeError, Request};
use exc_types::SubscribeTickers;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to subscribe tickers.
#[derive(Debug, Default)]
pub struct MakeTickersOptions {}

/// Make a service to subscribe tickers.
pub trait MakeTickers {
    /// Service to subscribe tickers.
    type Service: ExcService<SubscribeTickers>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to subscribe tickers.
    fn make_tickers(&mut self, options: MakeTickersOptions) -> Self::Future;
}

impl<M> MakeTickers for M
where
    M: MakeService<
        MakeTickersOptions,
        SubscribeTickers,
        Response = <SubscribeTickers as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: ExcService<SubscribeTickers>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_tickers(&mut self, options: MakeTickersOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}
