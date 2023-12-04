use exc_core::{
    types::{
        trading::PlaceOrderOptions, Canceled, OrderStream, OrderUpdate, Placed, SubscribeOrders,
    },
    Str,
};
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use tower::ServiceExt;

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
    <S as ExcService<PlaceOrder>>::Future: Send,
    <S as ExcService<CancelOrder>>::Future: Send,
{
    /// Place an order with options.
    fn place_with_opts(
        &mut self,
        place: &Place,
        opts: &PlaceOrderOptions,
    ) -> BoxFuture<'_, crate::Result<Placed>> {
        let req = (*place).into_request(opts);
        ServiceExt::<PlaceOrder>::oneshot(self.as_service(), req)
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
        ServiceExt::<CancelOrder>::oneshot(self.as_service(), CancelOrder::new(inst, id.clone()))
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
            self.as_service(),
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
        ServiceExt::<SubscribeOrders>::oneshot(self.as_service(), SubscribeOrders::new(inst))
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn boxed_trading<'a, S>(svc: S) -> Box<dyn TradingService + 'a>
    where
        S: TradingService + 'a,
    {
        Box::new(svc)
    }

    #[cfg(feature = "okx")]
    #[tokio::test]
    async fn test_box_trading() {
        let okx = crate::Okx::endpoint().connect_exc();
        std::hint::black_box(boxed_trading(okx));
    }

    #[allow(dead_code)]
    fn boxed_check_order<'a, S>(svc: S) -> Box<dyn CheckOrderService + 'a>
    where
        S: CheckOrderService + 'a,
    {
        Box::new(svc)
    }

    #[cfg(feature = "okx")]
    #[tokio::test]
    async fn test_box_check_order() {
        let okx = crate::Okx::endpoint().connect_exc();
        std::hint::black_box(boxed_check_order(okx));
    }

    #[allow(dead_code)]
    fn boxed_subscribe_order<'a, S>(svc: S) -> Box<dyn SubscribeOrdersService + 'a>
    where
        S: SubscribeOrdersService + 'a,
    {
        Box::new(svc)
    }

    #[cfg(feature = "okx")]
    #[tokio::test]
    async fn test_box_subscribe_order() {
        let okx = crate::Okx::endpoint().connect_exc();
        std::hint::black_box(boxed_subscribe_order(okx));
    }
}
