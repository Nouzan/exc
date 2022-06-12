use futures::{future::BoxFuture, FutureExt};
use tower::ServiceExt;

use crate::{
    types::trading::{CancelOrder, OrderId, Place, PlaceOrder},
    ExchangeError,
};

use super::ExchangeService;

/// Trading service.
pub trait TradingService: ExchangeService<PlaceOrder> + ExchangeService<CancelOrder> {
    /// Place an order.
    fn place(&mut self, inst: &str, place: &Place) -> BoxFuture<'_, Result<OrderId, ExchangeError>>
    where
        Self: Sized + Send,
        <Self as ExchangeService<PlaceOrder>>::Future: std::marker::Send,
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
        <Self as ExchangeService<CancelOrder>>::Future: std::marker::Send,
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
