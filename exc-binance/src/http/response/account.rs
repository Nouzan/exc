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
