use exc_core::types::ticker::Ticker;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use time::OffsetDateTime;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct OkxTicker {
    pub(super) inst_type: String,
    pub(super) inst_id: String,
    pub(super) last: Decimal,
    pub(super) last_sz: Decimal,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) ask_px: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) ask_sz: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) bid_px: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) bid_sz: Option<Decimal>,
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
        let bid_size = ti.bid_px.and(ti.bid_sz);
        let ask_size = ti.ask_px.and(ti.ask_sz);
        Self {
            ts: ti.ts,
            last: ti.last,
            size: ti.last_sz,
            buy: None,
            bid: ti.bid_px,
            ask: ti.ask_px,
            bid_size,
            ask_size,
        }
    }
}
