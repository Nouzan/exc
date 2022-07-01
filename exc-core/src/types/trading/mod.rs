/// Place: the order builder.
pub mod place;

/// Order.
pub mod order;

use futures::{future::BoxFuture, stream::BoxStream};
pub use order::{Order, OrderId, OrderKind, OrderState, OrderStatus, TimeInForce};
pub use place::Place;

use crate::{ExchangeError, Request};

/// Place order.
#[derive(Debug, Clone)]
pub struct PlaceOrder {
    /// Instrument.
    pub instrument: String,
    /// Place.
    pub place: Place,
}

impl Request for PlaceOrder {
    type Response = BoxFuture<'static, Result<OrderId, ExchangeError>>;
}

/// Cancel order.
#[derive(Debug, Clone)]
pub struct CancelOrder {
    /// Instrument.
    pub instrument: String,
    /// Id.
    pub id: OrderId,
}

impl Request for CancelOrder {
    type Response = BoxFuture<'static, Result<(), ExchangeError>>;
}

/// Get order.
#[derive(Debug, Clone)]
pub struct GetOrder {
    /// Instrument.
    pub instrument: String,
    /// Id.
    pub id: OrderId,
}

impl Request for GetOrder {
    type Response = BoxFuture<'static, Result<Order, ExchangeError>>;
}

/// Orders Stream.
pub type OrderStream = BoxStream<'static, Result<Order, ExchangeError>>;

/// Subscribe to order updates.
#[derive(Debug, Clone)]
pub struct SubscribeOrders {
    /// Instrument.
    pub instrument: String,
}

impl Request for SubscribeOrders {
    type Response = OrderStream;
}
