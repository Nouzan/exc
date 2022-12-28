use exc_core::{
    types::{trading::PlaceOrderOptions, SubscribeOrders},
    ExcMut, Request, Str,
};
use futures::{future::TryFlatten, TryFutureExt};
use tower::{util::Oneshot, ServiceExt};

use crate::core::types::trading::{CancelOrder, GetOrder, OrderId, Place, PlaceOrder};

use crate::ExcService;

type Fut<'a, S, R> = TryFlatten<Oneshot<ExcMut<'a, S>, R>, <R as Request>::Response>;

/// Trading service.
pub trait TradingService: ExcService<PlaceOrder> + ExcService<CancelOrder> + Sized {
    /// Place an order with options.
    fn place_with_opts(
        &mut self,
        place: &Place,
        opts: &PlaceOrderOptions,
    ) -> Fut<'_, Self, PlaceOrder> {
        let req = (*place).into_request(opts);
        ServiceExt::<PlaceOrder>::oneshot(ExcService::<PlaceOrder>::as_service_mut(self), req)
            .try_flatten()
    }
    /// Place an order.
    fn place(
        &mut self,
        inst: &str,
        place: &Place,
        client_id: Option<&str>,
    ) -> Fut<'_, Self, PlaceOrder> {
        self.place_with_opts(
            place,
            PlaceOrderOptions::new(inst).with_client_id(client_id),
        )
    }

    /// Cancel an order.
    fn cancel(&mut self, inst: &str, id: &OrderId) -> Fut<'_, Self, CancelOrder> {
        ServiceExt::<CancelOrder>::oneshot(
            ExcService::<CancelOrder>::as_service_mut(self),
            CancelOrder {
                instrument: inst.to_string(),
                id: id.clone(),
            },
        )
        .try_flatten()
    }
}

impl<S> TradingService for S where S: ExcService<PlaceOrder> + ExcService<CancelOrder> {}

/// Check order service.
pub trait CheckOrderService: ExcService<GetOrder> + Sized {
    /// Check the current status of an order.
    fn check(&mut self, inst: &str, id: &OrderId) -> Fut<'_, Self, GetOrder> {
        ServiceExt::oneshot(
            ExcService::<GetOrder>::as_service_mut(self),
            GetOrder {
                instrument: Str::new(inst),
                id: id.clone(),
            },
        )
        .try_flatten()
    }
}

impl<S> CheckOrderService for S where S: ExcService<GetOrder> {}

/// Subscribe orders service.
pub trait SubscribeOrdersService: ExcService<SubscribeOrders> + Sized {
    /// Subscribe orders.
    fn subscribe_orders(&mut self, inst: &str) -> Oneshot<ExcMut<'_, Self>, SubscribeOrders> {
        ServiceExt::<SubscribeOrders>::oneshot(
            self.as_service_mut(),
            SubscribeOrders {
                instrument: inst.to_string(),
            },
        )
    }
}

impl<S> SubscribeOrdersService for S where S: ExcService<SubscribeOrders> {}
