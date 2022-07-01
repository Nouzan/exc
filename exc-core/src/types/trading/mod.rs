/// Place: the order builder.
pub mod place;

/// Order.
pub mod order;

use std::fmt;

use futures::{future::BoxFuture, stream::BoxStream};
pub use order::{Order, OrderId, OrderKind, OrderState, OrderStatus, OrderTrade, TimeInForce};
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
#[derive(Clone)]
pub struct Placed {
    /// Order id.
    pub id: OrderId,
    /// The placed order.
    pub order: Option<Order>,
    /// Timestamp.
    pub ts: OffsetDateTime,
}

impl fmt::Debug for Placed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Placed")
            .field("ts", &self.ts.to_string())
            .field("id", &self.id.as_str())
            .field("order", &self.order)
            .finish()
    }
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
#[derive(Clone)]
pub struct Cancelled {
    /// The placed order.
    pub order: Option<Order>,
    /// Timestamp.
    pub ts: OffsetDateTime,
}

impl fmt::Debug for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cancelled")
            .field("ts", &self.ts.to_string())
            .field("order", &self.order)
            .finish()
    }
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
#[derive(Clone)]
pub struct OrderUpdate {
    /// Timestamp.
    pub ts: OffsetDateTime,
    /// Order.
    pub order: Order,
}

impl fmt::Debug for OrderUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderUpdate")
            .field("ts", &self.ts.to_string())
            .field("order", &self.order)
            .finish()
    }
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
