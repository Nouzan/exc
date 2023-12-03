use exc_service::{ExcService, ExchangeError, Request};
use exc_types::SubscribeOrders;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to subscribe orders.
#[derive(Debug, Default)]
pub struct MakeSubscribeOrdersOptions {}

/// Make a service to subscribe orders.
pub trait MakeSubscribeOrders {
    /// Service to subscribe orders.
    type Service: ExcService<SubscribeOrders>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to subscribe orders.
    fn make_subscribe_orders(&mut self, options: MakeSubscribeOrdersOptions) -> Self::Future;

    /// Convert to a [`Service`](tower_service::Service).
    fn as_make_subscribe_orders_service(&mut self) -> AsService<'_, Self>
    where
        Self: Sized,
    {
        AsService { make: self }
    }
}

impl<M> MakeSubscribeOrders for M
where
    M: MakeService<
        MakeSubscribeOrdersOptions,
        SubscribeOrders,
        Response = <SubscribeOrders as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: ExcService<SubscribeOrders>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_subscribe_orders(&mut self, options: MakeSubscribeOrdersOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}

crate::create_as_service!(
    MakeSubscribeOrders,
    MakeSubscribeOrdersOptions,
    make_subscribe_orders,
    "Service returns by [`MakeSubscribeOrders::as_make_subscribe_orders_service`]."
);
