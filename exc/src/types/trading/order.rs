use std::{fmt, sync::Arc};

use positions::{Position, Representation};
use rust_decimal::Decimal;

use super::place::Place;

/// Order types.
#[derive(Debug, Clone, Copy)]
pub enum OrderKind {
    /// Market.
    Market,
    /// Limit.
    Limit(Decimal),
}

/// Order Status.
#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    /// Placing.
    Placing,
    /// Pending.
    Pending,
    /// Finished.
    Finished,
}

/// Order State.
pub struct OrderState<Rep> {
    filled: Position<Rep, Decimal>,
    status: OrderStatus,
}

impl<Rep: Representation> fmt::Debug for OrderState<Rep> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderState")
            .field("filled", &self.filled)
            .field("status", &self.status)
            .finish()
    }
}

/// Order.
pub struct Order<Rep> {
    id: OrderId,
    target: Place,
    state: OrderState<Rep>,
}

impl<Rep: Representation> fmt::Debug for Order<Rep> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Order")
            .field("id", &self.id.inner)
            .field("target", &self.target)
            .field("state", &self.state)
            .finish()
    }
}

/// Order identity.
#[derive(Debug, Clone)]
pub struct OrderId {
    inner: Arc<String>,
}
