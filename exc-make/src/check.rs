use exc_service::{ExcService, ExchangeError, Request};
use exc_types::GetOrder;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to check orders.
#[derive(Debug)]
pub struct MakeCheckOrderOptions {}

/// Make a service to check orders.
pub trait MakeCheckOrder {
    /// Service to check orders.
    type Service: ExcService<GetOrder>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to check orders.
    fn make_check_order(&mut self, options: MakeCheckOrderOptions) -> Self::Future;
}

impl<M> MakeCheckOrder for M
where
    M: MakeService<
        MakeCheckOrderOptions,
        GetOrder,
        Response = <GetOrder as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: ExcService<GetOrder>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_check_order(&mut self, options: MakeCheckOrderOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}
