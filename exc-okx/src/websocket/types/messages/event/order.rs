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
    #[serde_as(as = "NoneAsEmptyString")]
    pub ccy: Option<String>,
    /// Order id.
    pub ord_id: String,
    /// Client id.
    #[serde_as(as = "NoneAsEmptyString")]
    pub cl_ord_id: Option<String>,
    /// Tag.
    #[serde_as(as = "NoneAsEmptyString")]
    pub tag: Option<String>,
    /// Price.
    #[serde_as(as = "NoneAsEmptyString")]
    pub px: Option<Decimal>,
    /// Size.
    pub sz: Decimal,
    /// Notional usd.
    #[serde_as(as = "NoneAsEmptyString")]
    pub notional_usd: Option<Decimal>,
    /// Order type.
    pub ord_type: String,
    /// Side.
    pub side: String,
    /// Position side.
    #[serde_as(as = "NoneAsEmptyString")]
    pub pos_side: Option<String>,
    /// Trade mode.
    pub td_mode: String,
    /// "Tgt" currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub tgt_ccy: Option<String>,
    /// Fill price.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_px: Option<Decimal>,
    /// Trade id.
    #[serde_as(as = "NoneAsEmptyString")]
    pub trade_id: Option<String>,
    /// Fill size.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_sz: Option<Decimal>,
    /// Fill time.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    #[serde(rename = "fillTime")]
    pub fill_ts: Option<OffsetDateTime>,
    /// Fill fee.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_fee: Option<Decimal>,
    /// Fill fee currency.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_fee_ccy: Option<String>,
    /// Execute type.
    #[serde_as(as = "NoneAsEmptyString")]
    pub exec_type: Option<String>,
    /// Total filled size.
    pub acc_fill_sz: Decimal,
    /// Filled usd.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_notional_usd: Option<Decimal>,
    /// Average price.
    pub avg_px: Decimal,
    /// State.
    pub state: String,
    /// Leverage.
    #[serde_as(as = "NoneAsEmptyString")]
    pub lever: Option<Decimal>,
    /// Take-profit trigger price.
    #[serde(rename = "tpTriggerPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub take_profit_trigger_price: Option<Decimal>,
    /// Take-profit trigger type.
    #[serde(rename = "tpTriggerPxType")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub take_profit_trigger_type: Option<String>,
    /// Take-profit price.
    #[serde(rename = "tpOrdPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub take_profit_price: Option<Decimal>,
    /// Stop-loss trigger price.
    #[serde(rename = "slTriggerPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_trigger_price: Option<Decimal>,
    /// Stop-loss trigger type.
    #[serde(rename = "slTriggerPxType")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_trigger_type: Option<String>,
    /// Stop-loss price.
    #[serde(rename = "slOrdPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_price: Option<Decimal>,
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
    #[serde_as(as = "NoneAsEmptyString")]
    pub source: Option<i64>,
    /// Cancel source.
    #[serde_as(as = "NoneAsEmptyString")]
    pub cancel_source: Option<i64>,
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
    #[serde_as(as = "NoneAsEmptyString")]
    pub amend_result: Option<i64>,
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
