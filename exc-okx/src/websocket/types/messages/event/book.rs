use exc_core::types::BidAsk;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

type Depth = [Decimal; 4];

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct OkxBook {
    pub(super) bids: Vec<Depth>,
    pub(super) asks: Vec<Depth>,
    #[serde(with = "crate::utils::timestamp_serde")]
    pub(super) ts: OffsetDateTime,
    pub(super) checksum: Option<i64>,
    pub(super) prev_seq_id: Option<i64>,
    pub(super) seq_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) enum Side {
    Buy,
    Sell,
}

impl From<OkxBook> for BidAsk {
    fn from(value: OkxBook) -> Self {
        let bid = value.bids.first().map(|depth| (depth[0], depth[1]));
        let ask = value.asks.first().map(|depth| (depth[0], depth[1]));
        Self {
            ts: value.ts,
            bid,
            ask,
        }
    }
}
