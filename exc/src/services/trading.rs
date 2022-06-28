use futures::{future::BoxFuture, FutureExt};
use tower::ServiceExt;

use crate::{
    types::trading::{CancelOrder, GetOrder, Order, OrderId, Place, PlaceOrder},
    ExchangeError,
};

use crate::ExcService;

/// Trading service.
pub trait TradingService: ExcService<PlaceOrder> + ExcService<CancelOrder> {
    /// Place an order.
    fn place(&mut self, inst: &str, place: &Place) -> BoxFuture<'_, Result<OrderId, ExchangeError>>
    where
        Self: Sized + Send,
        <Self as ExcService<PlaceOrder>>::Future: Send,
    {
        let resp = ServiceExt::<PlaceOrder>::oneshot(
            ExcService::<PlaceOrder>::as_service_mut(self),
            PlaceOrder {
                instrument: inst.to_string(),
                place: *place,
            },
        );
        async move { resp.await?.await }.boxed()
    }

    /// Cancel an order.
    fn cancel(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, Result<(), ExchangeError>>
    where
        Self: Sized + Send,
        <Self as ExcService<CancelOrder>>::Future: Send,
    {
        let resp = ServiceExt::<CancelOrder>::oneshot(
            ExcService::<CancelOrder>::as_service_mut(self),
            CancelOrder {
                instrument: inst.to_string(),
                id: id.clone(),
            },
        );
        async move { resp.await?.await }.boxed()
    }
}

impl<S> TradingService for S where S: ExcService<PlaceOrder> + ExcService<CancelOrder> {}

/// Check order service.
pub trait CheckOrderService: ExcService<GetOrder> {
    /// Check the current status of an order.
    fn check(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, Result<Order, ExchangeError>>
    where
        Self: Sized + Send,
        Self::Future: Send,
    {
        let resp = ServiceExt::oneshot(
            ExcService::<GetOrder>::as_service_mut(self),
            GetOrder {
                instrument: inst.to_string(),
                id: id.clone(),
            },
        );
        async move { resp.await?.await }.boxed()
    }
}

impl<S> CheckOrderService for S where S: ExcService<GetOrder> {}
