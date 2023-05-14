use exc_core::{
    types::{
        trading::PlaceOrderOptions, Canceled, OrderStream, OrderUpdate, Placed, SubscribeOrders,
    },
    Str,
};
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use tower::{Service, ServiceExt};

use crate::core::types::trading::{CancelOrder, GetOrder, OrderId, Place, PlaceOrder};

use crate::ExcService;

/// Trading service.
pub trait TradingService {
    /// Place an order with options.
    fn place_with_opts(
        &mut self,
        place: &Place,
        opts: &PlaceOrderOptions,
    ) -> BoxFuture<'_, crate::Result<Placed>>;

    /// Place an order.
    fn place(
        &mut self,
        inst: &str,
        place: &Place,
        client_id: Option<&str>,
    ) -> BoxFuture<'_, crate::Result<Placed>> {
        self.place_with_opts(
            place,
            PlaceOrderOptions::new(inst).with_client_id(client_id),
        )
    }

    /// Cancel an order.
    fn cancel(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, crate::Result<Canceled>>;
}

impl<S> TradingService for S
where
    S: ExcService<PlaceOrder> + ExcService<CancelOrder> + Send,
    <S as Service<PlaceOrder>>::Future: Send,
    <S as Service<CancelOrder>>::Future: Send,
{
    /// Place an order with options.
    fn place_with_opts(
        &mut self,
        place: &Place,
        opts: &PlaceOrderOptions,
    ) -> BoxFuture<'_, crate::Result<Placed>> {
        let req = (*place).into_request(opts);
        ServiceExt::<PlaceOrder>::oneshot(ExcService::<PlaceOrder>::as_service_mut(self), req)
            .try_flatten()
            .boxed()
    }
    /// Place an order.
    fn place(
        &mut self,
        inst: &str,
        place: &Place,
        client_id: Option<&str>,
    ) -> BoxFuture<'_, crate::Result<Placed>> {
        self.place_with_opts(
            place,
            PlaceOrderOptions::new(inst).with_client_id(client_id),
        )
    }

    /// Cancel an order.
    fn cancel(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, crate::Result<Canceled>> {
        ServiceExt::<CancelOrder>::oneshot(
            ExcService::<CancelOrder>::as_service_mut(self),
            CancelOrder {
                instrument: inst.to_string(),
                id: id.clone(),
            },
        )
        .try_flatten()
        .boxed()
    }
}

/// Check order service.
pub trait CheckOrderService {
    /// Check the current status of an order.
    fn check(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, crate::Result<OrderUpdate>>;
}

impl<S> CheckOrderService for S
where
    S: ExcService<GetOrder> + Send,
    S::Future: Send,
{
    fn check(&mut self, inst: &str, id: &OrderId) -> BoxFuture<'_, crate::Result<OrderUpdate>> {
        ServiceExt::oneshot(
            ExcService::<GetOrder>::as_service_mut(self),
            GetOrder {
                instrument: Str::new(inst),
                id: id.clone(),
            },
        )
        .try_flatten()
        .boxed()
    }
}

/// Subscribe orders service.
pub trait SubscribeOrdersService {
    /// Subscribe orders.
    fn subscribe_orders(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<OrderStream>>;
}

impl<S> SubscribeOrdersService for S
where
    S: ExcService<SubscribeOrders> + Send,
    S::Future: Send,
{
    fn subscribe_orders(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<OrderStream>> {
        ServiceExt::<SubscribeOrders>::oneshot(
            self.as_service_mut(),
            SubscribeOrders {
                instrument: inst.to_string(),
            },
        )
        .boxed()
    }
}
