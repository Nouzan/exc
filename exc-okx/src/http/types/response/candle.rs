use rust_decimal::Decimal;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

/// Candle.
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Candle(
    #[serde_as(as = "DisplayFromStr")] u64,
    #[serde_as(as = "DisplayFromStr")] Decimal,
    #[serde_as(as = "DisplayFromStr")] Decimal,
    #[serde_as(as = "DisplayFromStr")] Decimal,
    #[serde_as(as = "DisplayFromStr")] Decimal,
    #[serde_as(as = "DisplayFromStr")] Decimal,
    #[serde_as(as = "DisplayFromStr")] Decimal,
);
