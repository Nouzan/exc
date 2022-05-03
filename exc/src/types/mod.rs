/// Ticker.
pub mod ticker;

/// Candle.
pub mod candle;

/// Subscriptions.
pub mod subscriptions;

/// Request trait.
pub trait Request {
    /// Response.
    type Response;
}
