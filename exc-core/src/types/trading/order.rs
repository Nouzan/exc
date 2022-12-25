use std::collections::HashMap;

use positions::prelude::Str;
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
    /// Trade.
    pub trade: Option<OrderTrade>,
}

impl Order {
    /// Create a new [`Order`].
    pub fn new(id: OrderId, target: Place) -> Self {
        Self {
            id,
            target,
            state: OrderState::default(),
            trade: None,
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
    inner: Str,
}

impl From<String> for OrderId {
    fn from(inner: String) -> Self {
        Self {
            inner: Str::new(inner),
        }
    }
}

impl OrderId {
    /// Convert to [`&str`]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

/// Order trade.
#[derive(Debug, Clone)]
pub struct OrderTrade {
    /// Price.
    pub price: Decimal,
    /// Size.
    pub size: Decimal,
    /// Fee.
    pub fee: Decimal,
    /// Fee asset.
    pub fee_asset: Option<String>,
}
