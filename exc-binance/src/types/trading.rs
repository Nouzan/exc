use serde::{Deserialize, Serialize};

/// Order side.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    /// Buy.
    Buy,
    /// Sell.
    Sell,
}

/// Position side.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionSide {
    /// Long.
    Long,
    /// Short.
    Short,
    /// Both.
    Both,
}

/// Time-in-force.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    /// Good-Till-Cancel.
    Gtc,
    /// Immdiate-Or-Cancel.
    Ioc,
    /// Fill-Or-Kill.
    Fok,
    /// Good-Till-Cancel (Post-Only)
    Gtx,
}
