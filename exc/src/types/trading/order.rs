use rust_decimal::Decimal;

/// Order types.
pub enum OrderKind {
    /// Market.
    Market,
    /// Limit.
    Limit(Decimal),
}
