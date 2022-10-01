use exc_core::types::ticker::Ticker;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct OkxTicker {
    pub(super) inst_type: String,
    pub(super) inst_id: String,
    pub(super) last: Decimal,
    pub(super) last_sz: Decimal,
    pub(super) ask_px: Decimal,
    pub(super) ask_sz: Decimal,
    pub(super) bid_px: Decimal,
    pub(super) bid_sz: Decimal,
    pub(super) open_24h: Decimal,
    pub(super) high_24h: Decimal,
    pub(super) low_24h: Decimal,
    pub(super) vol_ccy_24h: Decimal,
    pub(super) vol_24h: Decimal,
    pub(super) sod_utc_0: Decimal,
    pub(super) sod_utc_8: Decimal,
    #[serde(with = "crate::utils::timestamp_serde")]
    pub(super) ts: OffsetDateTime,
}

impl From<OkxTicker> for Ticker {
    fn from(ti: OkxTicker) -> Self {
        Self {
            ts: ti.ts,
            last: ti.last,
            size: ti.last_sz,
            buy: None,
            bid: Some(ti.bid_px),
            ask: Some(ti.ask_px),
            bid_size: Some(ti.bid_sz),
            ask_size: Some(ti.ask_sz),
        }
    }
}
