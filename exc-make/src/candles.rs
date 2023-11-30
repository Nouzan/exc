use exc_service::{ExcService, ExchangeError, Request};
use exc_types::QueryCandles;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to fetch candles.
#[derive(Debug, Default)]
pub struct MakeFetchCandlesOptions {}

/// Make a service to subscribe instruments.
pub trait MakeFetchCandles {
    /// Service to fetch candles.
    type Service: ExcService<QueryCandles>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to fetch candles.
    fn make_fetch_candles(&mut self, options: MakeFetchCandlesOptions) -> Self::Future;
}

impl<M> MakeFetchCandles for M
where
    M: MakeService<
        MakeFetchCandlesOptions,
        QueryCandles,
        Response = <QueryCandles as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: ExcService<QueryCandles>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_fetch_candles(&mut self, options: MakeFetchCandlesOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}
