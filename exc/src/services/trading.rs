use exc_core::{
    types::{trading::PlaceOrderOptions, Cancelled, OrderUpdate, Placed, SubscribeOrders},
    ExcMut,
};
use futures::{future::BoxFuture, FutureExt};
use tower::{util::Oneshot, Service, ServiceExt};

use crate::{
    types::trading::{CancelOrder, GetOrder, OrderId, Place, PlaceOrder},
    ExchangeError,
};

use crate::ExcService;

/// Trading service.
pub trait TradingService: ExcService<PlaceOrder> + ExcService<CancelOrder> {
    /// Place an order with options.
    fn place_with_opts(
        &mut self,
        place: &Place,
        opts: &PlaceOrderOptions,
    ) -> BoxFuture<'_, Result<Placed, ExchangeError>>
    where
        Self: Sized + Send,
        <Self as Service<PlaceOrder>>::Future: Send,
    {
        let req = (*place).into_request(opts);
        let resp =
            ServiceExt::<PlaceOrder>::oneshot(ExcService::<PlaceOrder>::as_service_mut(self), req);
        async move { resp.await?.await }.boxed()
    }
    /// Place an order.
    fn place(
        &mut self,
        inst: &str,
        place: &Place,
        client_id: Option<&str>,
    ) -> BoxFuture<'_, Result<Placed, ExchangeError>>
    where
        Self: Sized + Send,
        <Self as Service<PlaceOrder>>::Future: Send,
    {
        self.place_with_opts(
            place,
            PlaceOrderOptions::new(inst).with_client_id(client_id),
        )
    }

    /// Cancel an order.
    fn cancel(
        &mut self,
        inst: &str,
        id: &OrderId,
    ) -> BoxFuture<'_, Result<Cancelled, ExchangeError>>
    where
        Self: Sized + Send,
        <Self as Service<CancelOrder>>::Future: Send,
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
    fn check(
        &mut self,
        inst: &str,
        id: &OrderId,
    ) -> BoxFuture<'_, Result<OrderUpdate, ExchangeError>>
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

/// Subscribe orders service.
pub trait SubscribeOrdersService: ExcService<SubscribeOrders> {
    /// Subscribe orders.
    fn subscribe_orders(&mut self, inst: &str) -> Oneshot<ExcMut<'_, Self>, SubscribeOrders>
    where
        Self: Sized,
    {
        ServiceExt::<SubscribeOrders>::oneshot(
            self.as_service_mut(),
            SubscribeOrders {
                instrument: inst.to_string(),
            },
        )
    }
}

impl<S> SubscribeOrdersService for S where S: ExcService<SubscribeOrders> {}
