use exc_service::{ExchangeError, Request, SendExcService};
use exc_types::QueryCandles;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to fetch candles.
#[derive(Debug, Default)]
pub struct MakeFetchCandlesOptions {
    /// Rate limit.
    pub rate_limit: Option<(u64, std::time::Duration)>,
    /// Batch limit.
    pub batch_limit: Option<usize>,
}

/// Make a service to subscribe instruments.
pub trait MakeFetchCandles {
    /// Service to fetch candles.
    type Service: SendExcService<QueryCandles>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to fetch candles.
    fn make_fetch_candles(&mut self, options: MakeFetchCandlesOptions) -> Self::Future;

    /// Convert to a [`Service`](tower_service::Service).
    fn as_make_fetch_candles_service(&mut self) -> AsService<'_, Self>
    where
        Self: Sized,
    {
        AsService { make: self }
    }
}

impl<M> MakeFetchCandles for M
where
    M: MakeService<
        MakeFetchCandlesOptions,
        QueryCandles,
        Response = <QueryCandles as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: SendExcService<QueryCandles>,
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

crate::create_as_service!(
    MakeFetchCandles,
    MakeFetchCandlesOptions,
    make_fetch_candles,
    "Service returns by [`MakeFetchCandles::as_make_fetch_candles_service`]."
);
