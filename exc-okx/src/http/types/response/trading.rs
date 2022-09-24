use rust_decimal::Decimal;
use serde::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};

/// Order detail.
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct OrderDetail {
    /// Inst type.
    #[serde(rename = "instType")]
    pub instrument_type: String,
    /// Inst.
    #[serde(rename = "instId")]
    pub instrument: String,
    /// Currency.
    #[serde(rename = "ccy")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub currency: Option<String>,
    /// Order id.
    #[serde(rename = "ordId")]
    pub order_id: String,
    /// Client id.
    #[serde(rename = "clOrdId")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub client_id: Option<String>,
    /// Tag.
    pub tag: String,
    /// Price.
    #[serde(rename = "px")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub price: Option<Decimal>,
    /// Size.
    #[serde(rename = "sz")]
    pub size: Decimal,
    /// Pnl.
    pub pnl: Decimal,
    /// Order type.
    #[serde(rename = "ordType")]
    pub order_type: String,
    /// Order side.
    pub side: String,
    /// Position side.
    #[serde(rename = "posSide")]
    pub position_side: String,
    /// Trade mode.
    #[serde(rename = "tdMode")]
    pub trade_mode: String,
    /// Filled size.
    #[serde(rename = "accFillSz")]
    pub filled_size: Decimal,
    /// Filled price.
    #[serde(rename = "fillPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub fill_price: Option<Decimal>,
    /// Trade id.
    #[serde(rename = "tradeId")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub trade_id: Option<String>,
    /// Last filled size.
    #[serde(rename = "fillSz")]
    pub fill_size: Decimal,
    #[serde(rename = "fillTime")]
    #[serde_as(as = "NoneAsEmptyString")]
    /// Last filled time.
    pub fill_time: Option<Decimal>,
    /// State.
    pub state: String,
    /// Average price.
    #[serde(rename = "avgPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub avg_price: Option<Decimal>,
    /// Leverage.
    #[serde_as(as = "NoneAsEmptyString")]
    pub lever: Option<Decimal>,
    /// Spt.
    #[serde(rename = "tpTriggerPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_profit_trigger: Option<Decimal>,
    /// Spl.
    #[serde(rename = "tpOrdPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_profit_limit: Option<Decimal>,
    /// Slt.
    #[serde(rename = "slTriggerPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_trigger: Option<Decimal>,
    /// Sll.
    #[serde(rename = "slOrdPx")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub stop_loss_limit: Option<Decimal>,
    #[serde(rename = "feeCcy")]
    #[serde_as(as = "NoneAsEmptyString")]
    /// Fee currency.
    pub fee_currency: Option<String>,
    /// Fee.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fee: Option<Decimal>,
    /// Rebate currency
    #[serde(rename = "rebateCcy")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub rebate_currency: Option<String>,
    /// Rebate
    #[serde_as(as = "NoneAsEmptyString")]
    pub rebate: Option<Decimal>,
    /// Category.
    #[serde_as(as = "NoneAsEmptyString")]
    pub category: Option<String>,
    /// Updated.
    #[serde(rename = "uTime")]
    pub updated_at: Decimal,
    /// Created.
    #[serde(rename = "cTime")]
    pub created_at: Decimal,
}
