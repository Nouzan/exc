use super::{order::TimeInForce, OrderKind};
use rust_decimal::Decimal;

/// A [`Place`] describes how exchange build an order, i.e. the order builder.
/// The sign of `size` representants the side of the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Place {
    /// Size.
    pub size: Decimal,
    /// Order types.
    pub kind: OrderKind,
}

impl Place {
    /// Create a new order placement (order builder) with default config.
    /// The sign of `size` representants the side of the order.
    pub fn with_size(size: Decimal) -> Self {
        Self {
            size,
            kind: OrderKind::Market,
        }
    }

    /// Convert to a limit order (with TIF set to GTC).
    pub fn limit(self, price: Decimal) -> Self {
        self.limit_with_tif(price, TimeInForce::default())
    }

    /// Convert tto a limit order with the given time-in-force option.
    pub fn limit_with_tif(mut self, price: Decimal, tif: TimeInForce) -> Self {
        self.kind = OrderKind::Limit(price, tif);
        self
    }

    /// Convert to a post-only order.
    pub fn post_only(mut self, price: Decimal) -> Self {
        self.kind = OrderKind::PostOnly(price);
        self
    }
}
