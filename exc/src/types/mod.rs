/// Ticker.
pub mod ticker;

/// Candle.
pub mod candle;

/// Subscriptions.
pub mod subscriptions;

/// Request and Response binding.
pub trait Request {
    /// Response type.
    type Response;
}
