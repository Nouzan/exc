/// Place: the order builder.
pub mod place;

/// Order.
pub mod order;

use futures::future::BoxFuture;
pub use order::{Order, OrderId, OrderKind};
pub use place::Place;

use crate::ExchangeError;

use super::Request;

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
