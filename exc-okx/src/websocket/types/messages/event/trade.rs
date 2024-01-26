use exc_core::types::Trade;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct OkxTrade {
    pub(super) inst_id: String,
    pub(super) trade_id: String,
    pub(super) px: Decimal,
    pub(super) sz: Decimal,
    pub(super) side: Side,
    pub(super) count: Decimal,
    #[serde(with = "crate::utils::timestamp_serde")]
    pub(super) ts: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) enum Side {
    Buy,
    Sell,
}

impl From<OkxTrade> for Trade {
    fn from(value: OkxTrade) -> Self {
        Self {
            ts: value.ts,
            price: value.px,
            size: value.sz,
            buy: matches!(value.side, Side::Buy),
        }
    }
}
