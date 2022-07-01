use std::{collections::HashMap, sync::Arc};

use rust_decimal::Decimal;

use super::place::Place;

/// Time in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    /// Good-Til-Cancelled.
    GoodTilCancelled,
    /// Fill-Or-Kill.
    FillOrKill,
    /// Immediate-Or-Cancel.
    ImmediateOrCancel,
}

impl Default for TimeInForce {
    fn default() -> Self {
        Self::GoodTilCancelled
    }
}

/// Order types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderKind {
    /// Market.
    Market,
    /// Limit.
    Limit(Decimal, TimeInForce),
    /// Post-Only.
    PostOnly(Decimal),
}

/// Order Status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    /// Pending.
    Pending,
    /// Finished.
    Finished,
    /// Unknown.
    Unknown,
}

/// Order State.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Fees.
    pub fees: HashMap<String, Decimal>,
}

impl Default for OrderState {
    fn default() -> Self {
        Self {
            filled: Decimal::ZERO,
            cost: Decimal::ONE,
            base_fee: Decimal::ZERO,
            quote_fee: Decimal::ZERO,
            status: OrderStatus::Pending,
            fees: HashMap::default(),
        }
    }
}

/// Order.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
