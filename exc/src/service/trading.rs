use futures::{future::BoxFuture, FutureExt};
use tower::ServiceExt;

use crate::{
    types::trading::{CancelOrder, OrderId, Place, PlaceOrder, GetOrder, Order},
    ExchangeError,
};

use super::ExchangeService;

/// Trading service.
pub trait TradingService: ExchangeService<PlaceOrder> + ExchangeService<CancelOrder> {
    /// Place an order.
    fn place(&mut self, inst: &str, place: &Place) -> BoxFuture<'_, Result<OrderId, ExchangeError>>
    where
        Self: Sized + Send,
        <Self as ExchangeService<PlaceOrder>>::Future: Send,
    {
        let resp = ServiceExt::<PlaceOrder>::oneshot(
            ExchangeService::<PlaceOrder>::as_service_mut(self),
            PlaceOrder {
                instrument: inst.to_string(),
                place: place.clone(),
            },
        );
        async move { Ok(resp.await?.await?) }.boxed()
    }

    /// Cancel an order.
    fn cancel(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, Result<(), ExchangeError>>
    where
        Self: Sized + Send,
        <Self as ExchangeService<CancelOrder>>::Future: Send,
    {
        let resp = ServiceExt::<CancelOrder>::oneshot(
            ExchangeService::<CancelOrder>::as_service_mut(self),
            CancelOrder {
                instrument: inst.to_string(),
                id: id.clone(),
            },
        );
        async move { Ok(resp.await?.await?) }.boxed()
    }
}

impl<S> TradingService for S where S: ExchangeService<PlaceOrder> + ExchangeService<CancelOrder> {}

/// Check order service.
pub trait CheckOrderService: ExchangeService<GetOrder>{
    /// Check the current status of an order.
    fn check(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, Result<Order, ExchangeError>>
    where
        Self: Sized + Send,
        Self::Future: Send,
    {
        let resp = ServiceExt::oneshot(
            ExchangeService::<GetOrder>::as_service_mut(self),
            GetOrder {
                instrument: inst.to_string(),
                id: id.clone(),
            },
        );
        async move { Ok(resp.await?.await?) }.boxed()
    }
}

impl<S> CheckOrderService for S where S: ExchangeService<GetOrder> {}
