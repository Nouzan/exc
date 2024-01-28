use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use time::OffsetDateTime;

/// Option Summary.
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OkxOptionSummary {
    /// Time.
    #[serde(with = "crate::utils::timestamp_serde")]
    pub ts: OffsetDateTime,
    /// Instrument id.
    pub inst_id: String,
    /// Instrument type.
    pub inst_type: String,
    /// Underlying.
    pub uly: String,
    /// Delta.
    #[serde_as(as = "NoneAsEmptyString")]
    pub delta: Option<Decimal>,
    /// Gamma.
    #[serde_as(as = "NoneAsEmptyString")]
    pub gamma: Option<Decimal>,
    /// Vega.
    #[serde_as(as = "NoneAsEmptyString")]
    pub vega: Option<Decimal>,
    /// Theta.
    #[serde_as(as = "NoneAsEmptyString")]
    pub theta: Option<Decimal>,
    /// Delta in BS mode.
    #[serde_as(as = "NoneAsEmptyString")]
    pub delta_b_s: Option<Decimal>,
    /// Gamma in BS mode.
    #[serde_as(as = "NoneAsEmptyString")]
    pub gamma_b_s: Option<Decimal>,
    /// Vega in BS mode.
    #[serde_as(as = "NoneAsEmptyString")]
    pub vega_b_s: Option<Decimal>,
    /// Theta in BS mode.
    #[serde_as(as = "NoneAsEmptyString")]
    pub theta_b_s: Option<Decimal>,
    /// Leverage.
    #[serde_as(as = "NoneAsEmptyString")]
    pub lever: Option<Decimal>,
    /// Mark Volatility.
    #[serde_as(as = "NoneAsEmptyString")]
    pub mark_vol: Option<Decimal>,
    /// Bid Volatility.
    #[serde_as(as = "NoneAsEmptyString")]
    pub bid_vol: Option<Decimal>,
    /// Ask Volatility.
    #[serde_as(as = "NoneAsEmptyString")]
    pub ask_vol: Option<Decimal>,
    /// Realized Volatility. (Not available for now)
    #[serde_as(as = "NoneAsEmptyString")]
    pub real_vol: Option<Decimal>,
    /// Implied Volatility of at-the-money option.
    #[serde_as(as = "NoneAsEmptyString")]
    pub vol_lv: Option<Decimal>,
    /// Forward price.
    #[serde_as(as = "NoneAsEmptyString")]
    pub fwd_px: Option<Decimal>,
}
