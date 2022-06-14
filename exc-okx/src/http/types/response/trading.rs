use rust_decimal::Decimal;
use serde::Deserialize;

/// Order detail.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderDetail {
    /// Inst type.
    #[serde(rename = "instType")]
    pub instrument_type: String,
    /// Inst.
    #[serde(rename = "instId")]
    pub instrument: String,
    /// Currency.
    #[serde(rename = "ccy", with = "serde_with::rust::string_empty_as_none")]
    pub currency: Option<String>,
    /// Order id.
    #[serde(rename = "ordId")]
    pub order_id: String,
    /// Client id.
    #[serde(rename = "clOrdId", with = "serde_with::rust::string_empty_as_none")]
    pub client_id: Option<String>,
    /// Tag.
    pub tag: String,
    /// Price.
    #[serde(rename = "px", with = "serde_with::rust::string_empty_as_none")]
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
    #[serde(rename = "fillPx", with = "serde_with::rust::string_empty_as_none")]
    pub fill_price: Option<Decimal>,
    /// Trade id.
    #[serde(rename = "tradeId", with = "serde_with::rust::string_empty_as_none")]
    pub trade_id: Option<String>,
    /// Last filled size.
    #[serde(rename = "fillSz")]
    pub fill_size: Decimal,
    #[serde(rename = "fillTime", with = "serde_with::rust::string_empty_as_none")]
    /// Last filled time.
    pub fill_time: Option<Decimal>,
    /// State.
    pub state: String,
    /// Average price.
    #[serde(rename = "avgPx", with = "serde_with::rust::string_empty_as_none")]
    pub avg_price: Option<Decimal>,
    /// Leverage.
    #[serde(with = "serde_with::rust::string_empty_as_none")]
    pub lever: Option<Decimal>,
    /// Spt.
    #[serde(
        rename = "tpTriggerPx",
        with = "serde_with::rust::string_empty_as_none"
    )]
    pub stop_profit_trigger: Option<Decimal>,
    /// Spl.
    #[serde(rename = "tpOrdPx", with = "serde_with::rust::string_empty_as_none")]
    pub stop_profit_limit: Option<Decimal>,
    /// Slt.
    #[serde(
        rename = "slTriggerPx",
        with = "serde_with::rust::string_empty_as_none"
    )]
    pub stop_loss_trigger: Option<Decimal>,
    /// Sll.
    #[serde(rename = "slOrdPx", with = "serde_with::rust::string_empty_as_none")]
    pub stop_loss_limit: Option<Decimal>,
    #[serde(rename = "feeCcy", with = "serde_with::rust::string_empty_as_none")]
    /// Fee currency.
    pub fee_currency: Option<String>,
    /// Fee.
    #[serde(with = "serde_with::rust::string_empty_as_none")]
    pub fee: Option<Decimal>,
    /// Rebate currency
    #[serde(rename = "rebateCcy", with = "serde_with::rust::string_empty_as_none")]
    pub rebate_currency: Option<String>,
    /// Rebate
    #[serde(with = "serde_with::rust::string_empty_as_none")]
    pub rebate: Option<Decimal>,
    /// Category.
    #[serde(with = "serde_with::rust::string_empty_as_none")]
    pub category: Option<String>,
    /// Updated.
    #[serde(rename = "uTime")]
    pub updated_at: Decimal,
    /// Created.
    #[serde(rename = "cTime")]
    pub created_at: Decimal,
}
