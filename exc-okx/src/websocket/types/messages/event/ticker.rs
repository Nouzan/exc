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
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) last: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) last_sz: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) ask_px: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) ask_sz: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) bid_px: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) bid_sz: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) open_24h: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) high_24h: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) low_24h: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) vol_ccy_24h: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) vol_24h: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) sod_utc_0: Option<Decimal>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub(super) sod_utc_8: Option<Decimal>,
    #[serde(with = "crate::utils::timestamp_serde")]
    pub(super) ts: OffsetDateTime,
}

impl From<OkxTicker> for Ticker {
    fn from(ti: OkxTicker) -> Self {
        let bid_size = ti.bid_px.and(ti.bid_sz);
        let ask_size = ti.ask_px.and(ti.ask_sz);
        let last = ti.last.or(ti.bid_px).or(ti.ask_px).unwrap_or(Decimal::ONE);
        let size = ti.last_sz.unwrap_or(Decimal::ZERO);
        Self {
            ts: ti.ts,
            last,
            size,
            buy: None,
            bid: ti.bid_px,
            ask: ti.ask_px,
            bid_size,
            ask_size,
        }
    }
}
