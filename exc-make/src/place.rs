use exc_service::{ExchangeError, Request, SendExcService};
use exc_types::PlaceOrder;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to place orders.
#[derive(Debug, Default)]
pub struct MakePlaceOrderOptions {}

/// Make a service to place orders.
pub trait MakePlaceOrder {
    /// Service to place orders.
    type Service: SendExcService<PlaceOrder>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to place orders.
    fn make_place_order(&mut self, options: MakePlaceOrderOptions) -> Self::Future;

    /// Convert to a [`Service`](tower_service::Service).
    fn as_make_place_order_service(&mut self) -> AsService<'_, Self>
    where
        Self: Sized,
    {
        AsService { make: self }
    }
}

impl<M> MakePlaceOrder for M
where
    M: MakeService<
        MakePlaceOrderOptions,
        PlaceOrder,
        Response = <PlaceOrder as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: SendExcService<PlaceOrder>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_place_order(&mut self, options: MakePlaceOrderOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}

crate::create_as_service!(
    MakePlaceOrder,
    MakePlaceOrderOptions,
    make_place_order,
    "Service returns by [`MakePlaceOrder::as_make_place_order_service`]."
);
