use std::sync::Arc;

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
    /// Pending.
    Pending,
    /// Finished.
    Finished,
}

/// Order State.
#[derive(Debug)]
pub struct OrderState {
    /// Filled size.
    pub filled: Decimal,
    /// Average cost.
    pub cost: Decimal,
    /// Fee or bonus in base currency.
    pub base_fee: Decimal,
    /// Fee or bonus in quote currency.
    pub quote_fee: Decimal,
    /// Status.
    pub status: OrderStatus,
}

impl Default for OrderState {
    fn default() -> Self {
        Self {
            filled: Decimal::ZERO,
            cost: Decimal::ONE,
            base_fee: Decimal::ZERO,
            quote_fee: Decimal::ZERO,
            status: OrderStatus::Pending,
        }
    }
}

/// Order.
#[derive(Debug)]
pub struct Order {
    /// Id.
    pub id: OrderId,
    /// The target of the order.
    pub target: Place,
    /// Current state.
    pub state: OrderState,
}

impl Order {
    /// Create a new [`Order`].
    pub fn new(id: OrderId, target: Place) -> Self {
        Self {
            id,
            target,
            state: OrderState::default(),
        }
    }

    /// Change the state.
    pub fn with_state(&mut self, state: OrderState) -> &mut Self {
        self.state = state;
        self
    }
}

/// Order identity.
#[derive(Debug, Clone)]
pub struct OrderId {
    inner: Arc<String>,
}

impl From<String> for OrderId {
    fn from(inner: String) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl OrderId {
    /// Convert to [`&str`]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}
