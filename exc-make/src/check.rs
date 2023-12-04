use exc_service::{ExchangeError, Request, SendExcService};
use exc_types::GetOrder;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to check orders.
#[derive(Debug, Default)]
pub struct MakeCheckOrderOptions {}

/// Make a service to check orders.
pub trait MakeCheckOrder {
    /// Service to check orders.
    type Service: SendExcService<GetOrder>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to check orders.
    fn make_check_order(&mut self, options: MakeCheckOrderOptions) -> Self::Future;

    /// Convert to a [`Service`](tower_service::Service).
    fn as_make_check_order_service(&mut self) -> AsService<'_, Self>
    where
        Self: Sized,
    {
        AsService { make: self }
    }
}

impl<M> MakeCheckOrder for M
where
    M: MakeService<
        MakeCheckOrderOptions,
        GetOrder,
        Response = <GetOrder as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: SendExcService<GetOrder>,
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

crate::create_as_service!(
    MakeCheckOrder,
    MakeCheckOrderOptions,
    make_check_order,
    "Service returns by [`MakeCheckOrder::as_make_check_order_service`]."
);
