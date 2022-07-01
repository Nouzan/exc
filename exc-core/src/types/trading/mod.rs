/// Place: the order builder.
pub mod place;

/// Order.
pub mod order;

use futures::{future::BoxFuture, stream::BoxStream};
pub use order::{Order, OrderId, OrderKind, OrderState, OrderStatus, TimeInForce};
pub use place::Place;
use time::OffsetDateTime;

use crate::{ExchangeError, Request};

/// Place order.
#[derive(Debug, Clone)]
pub struct PlaceOrder {
    /// Instrument.
    pub instrument: String,
    /// Place.
    pub place: Place,
    /// Client id.
    pub client_id: Option<String>,
}

/// Place order response.
#[derive(Debug, Clone)]
pub struct Placed {
    /// Order id.
    pub id: OrderId,
    /// The placed order.
    pub order: Option<Order>,
    /// Timestamp.
    pub ts: OffsetDateTime,
}

impl Request for PlaceOrder {
    type Response = BoxFuture<'static, Result<Placed, ExchangeError>>;
}

/// Cancel order.
#[derive(Debug, Clone)]
pub struct CancelOrder {
    /// Instrument.
    pub instrument: String,
    /// Id.
    pub id: OrderId,
}

/// Cancel order response.
#[derive(Debug, Clone)]
pub struct Cancelled {
    /// The placed order.
    pub order: Option<Order>,
    /// Timestamp.
    pub ts: OffsetDateTime,
}

impl Request for CancelOrder {
    type Response = BoxFuture<'static, Result<Cancelled, ExchangeError>>;
}

/// Get order.
#[derive(Debug, Clone)]
pub struct GetOrder {
    /// Instrument.
    pub instrument: String,
    /// Id.
    pub id: OrderId,
}

/// Order update.
#[derive(Debug, Clone)]
pub struct OrderUpdate {
    /// Timestamp.
    pub ts: OffsetDateTime,
    /// Order.
    pub order: Order,
}

impl Request for GetOrder {
    type Response = BoxFuture<'static, Result<OrderUpdate, ExchangeError>>;
}

/// Orders Stream.
pub type OrderStream = BoxStream<'static, Result<OrderUpdate, ExchangeError>>;

/// Subscribe to order updates.
#[derive(Debug, Clone)]
pub struct SubscribeOrders {
    /// Instrument.
    pub instrument: String,
}

impl Request for SubscribeOrders {
    type Response = OrderStream;
}
