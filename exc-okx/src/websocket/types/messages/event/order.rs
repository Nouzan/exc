use exc_core::types::OrderUpdate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};
use time::OffsetDateTime;

use crate::error::OkxError;

/// Okx Order message from websocket channel.
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OkxOrder {
    /// Instrument type.
    pub inst_type: String,
    /// Instrument name.
    pub inst_id: String,
    /// Margin currency.
    pub ccy: String,
    /// Order id.
    pub ord_id: String,
    /// Client id.
    pub cl_ord_id: String,
    /// Tag.
    pub tag: String,
    /// Price.
    pub px: Decimal,
    /// Size.
    pub sz: Decimal,
    /// Notional usd.
    pub notional_usd: Decimal,
    /// Order type.
    pub ord_type: String,
    /// Side.
    pub side: String,
    /// Position side.
    pub pos_side: String,
    /// Trade mode.
    pub td_mode: String,
    /// "Tgt" currency.
    pub tgt_ccy: String,
    /// Fill price.
    pub fill_px: Decimal,
    /// Trade id.
    pub trade_id: String,
    /// Fill size.
    pub fill_sz: Decimal,
    /// Fill time.
    #[serde(with = "crate::utils::timestamp_serde")]
    #[serde(rename = "fillTime")]
    pub fill_ts: OffsetDateTime,
    /// Fill fee.
    pub fill_fee: Decimal,
    /// Fill fee currency.
    pub fill_fee_ccy: String,
    /// Execute type.
    pub exec_type: String,
    /// Total filled size.
    pub acc_fill_sz: Decimal,
    /// Filled usd.
    pub fill_notional_usd: Decimal,
    /// Average price.
    pub avg_px: Decimal,
    /// State.
    pub state: String,
    /// Leverage.
    #[serde_as(as = "NoneAsEmptyString")]
    pub lever: Option<Decimal>,
    /// Take-profit trigger price.
    #[serde(rename = "tpTriggerPx")]
    pub take_profit_trigger_price: Decimal,
    /// Take-profit trigger type.
    #[serde(rename = "tpTriggerPxType")]
    pub take_profit_trigger_type: String,
    /// Take-profit price.
    #[serde(rename = "tpOrdPx")]
    pub take_profit_price: Decimal,
    /// Stop-loss trigger price.
    #[serde(rename = "slTriggerPx")]
    pub stop_loss_trigger_price: Decimal,
    /// Stop-loss trigger type.
    #[serde(rename = "slTriggerPxType")]
    pub stop_loss_trigger_type: String,
    /// Stop-loss price.
    #[serde(rename = "slOrdPx")]
    pub stop_loss_price: Decimal,
    /// Fee currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fee_ccy: Option<String>,
    /// Fee.
    pub fee: Decimal,
    /// Rebate currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub rebate_ccy: Option<String>,
    /// Rebate.
    pub rebate: Decimal,
    /// Profit and loss.
    pub pnl: Decimal,
    /// Source.
    #[serde_as(as = "DisplayFromStr")]
    pub source: i64,
    /// Cancel source.
    #[serde_as(as = "DisplayFromStr")]
    pub cancel_source: i64,
    /// Category.
    pub category: String,
    /// Update time.
    #[serde(with = "crate::utils::timestamp_serde")]
    #[serde(rename = "uTime")]
    pub update_ts: OffsetDateTime,
    /// Create time.
    #[serde(with = "crate::utils::timestamp_serde")]
    #[serde(rename = "cTime")]
    pub create_ts: OffsetDateTime,
    /// Request id.
    #[serde_as(as = "NoneAsEmptyString")]
    pub req_id: Option<String>,
    /// Amend result.
    pub amend_result: Decimal,
    /// Reduce only.
    pub reduce_only: String,
    /// Code.
    #[serde_as(as = "DisplayFromStr")]
    pub code: i64,
    /// Message.
    pub msg: String,
}

impl TryFrom<OkxOrder> for OrderUpdate {
    type Error = OkxError;

    fn try_from(order: OkxOrder) -> Result<Self, Self::Error> {
        todo!()
    }
}
