use super::OrderKind;
use rust_decimal::Decimal;

/// A [`Place`] describes how exchange build an order, i.e. the order builder.
/// The sign of `size` representants the side of the order.
pub struct Place {
    /// Size.
    pub size: Decimal,
    /// Order types.
    pub kind: OrderKind,
}

impl Place {
    /// Create a new order placement (order builder) with default config.
    /// The sign of `size` representants the side of the order.
    pub fn new(size: Decimal) -> Self {
        Self {
            size,
            kind: OrderKind::Market,
        }
    }

    /// Convert to a limit order placement.
    pub fn limit(mut self, price: Decimal) -> Self {
        self.kind = OrderKind::Limit(price);
        self
    }
}
