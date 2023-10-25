use exc_service::{ExcService, ExchangeError, Request};
use exc_types::CancelOrder;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to Cancel orders.
#[derive(Debug)]
pub struct MakeCancelOrderOptions {}

/// Make a service to cancel orders.
pub trait MakeCancelOrder {
    /// Service to cancel orders.
    type Service: ExcService<CancelOrder>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to cancel orders.
    fn make_cancel_order(&mut self, options: MakeCancelOrderOptions) -> Self::Future;
}

impl<M> MakeCancelOrder for M
where
    M: MakeService<
        MakeCancelOrderOptions,
        CancelOrder,
        Response = <CancelOrder as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: ExcService<CancelOrder>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_cancel_order(&mut self, options: MakeCancelOrderOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}
