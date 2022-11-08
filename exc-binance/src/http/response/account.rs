use rust_decimal::Decimal;
use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

/// Sub-account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccount {
    /// Email.
    pub email: String,
    /// Is freezed.
    pub is_freeze: bool,
    /// Created time.
    pub create_time: i64,
    /// Is managed sub account.
    pub is_managed_sub_account: bool,
    /// Is assest-management sub-account.
    pub is_asset_management_sub_account: bool,
}

/// Sub-accounts.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccounts {
    /// Sub-accounts.
    pub sub_accounts: Vec<SubAccount>,
}

impl TryFrom<Data> for SubAccounts {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::SubAccounts(data) => Ok(data),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}

/// A balance of a sub-account asset.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountBalance {
    /// Asset.
    pub asset: String,
    /// Free.
    #[serde(with = "rust_decimal::serde::float")]
    pub free: Decimal,
    /// Locked.
    #[serde(with = "rust_decimal::serde::float")]
    pub locked: Decimal,
}

/// Balances of sub-account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountBalances {
    /// Balances.
    pub balances: Vec<SubAccountBalance>,
}

impl TryFrom<Data> for SubAccountBalances {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::SubAccountBalances(data) => Ok(data),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}

/// A balance of a sub-account margin asset.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountMarginBalance {
    /// Asset.
    pub asset: String,
    /// Borrowed.
    pub borrowed: Decimal,
    /// Free.
    pub free: Decimal,
    /// Interset.
    pub interest: Decimal,
    /// Locked.
    pub locked: Decimal,
    /// Net.
    pub net_asset: Decimal,
}

/// Sub-account Margin Trade Coeff.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountMarginTradeCoeff {
    /// Force liquidation bar.
    pub force_liquidation_bar: Decimal,
    /// Margin call bar.
    pub margin_call_bar: Decimal,
    /// normal bar.
    pub normal_bar: Decimal,
}

/// Details of sub-account in margin account.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountMargin {
    /// Email.
    pub email: Option<String>,
    /// Margin level.
    pub margin_level: Decimal,
    /// Total asset in BTC.
    pub total_asset_of_btc: Decimal,
    /// Total liability in BTC.
    pub total_liability_of_btc: Decimal,
    /// Total net asset in BTC.
    pub total_net_asset_of_btc: Decimal,
    /// Margin trade coeff.
    pub margin_trade_coeff_vo: SubAccountMarginTradeCoeff,
    /// assets.
    pub margin_user_asset_vo_list: Vec<SubAccountMarginBalance>,
}

impl SubAccountMargin {
    /// Is margin account exist.
    pub fn is_exist(&self) -> bool {
        self.email.is_some()
    }
}

impl TryFrom<Data> for SubAccountMargin {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::SubAccountMargin(data) => Ok(data),
            Data::Error(err) => {
                if err.code == -3003 {
                    Ok(Self::default())
                } else {
                    Err(RestError::Api(err.code, err.message))
                }
            }
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
