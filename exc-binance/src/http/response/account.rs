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
    /// Assets.
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

/// Balance of an asset of the sub-account's  futures account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountFuturesBalance {
    /// Asset.
    pub asset: String,
    /// Initial margin.
    pub initial_margin: Decimal,
    /// Maintenance margin.
    pub maintenance_margin: Decimal,
    /// Margin Balance.
    pub margin_balance: Decimal,
    /// Max withdraw amount.
    pub max_withdraw_amount: Decimal,
    /// Open order initial margin.
    pub open_order_initial_margin: Decimal,
    /// Position initial margin.
    pub position_initial_margin: Decimal,
    /// Unrealized profit.
    pub unrealized_profit: Decimal,
    /// Wallet balance.
    pub wallet_balance: Decimal,
}

/// Details of sub-account in futures account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountFuturesInner {
    /// Email.
    pub email: String,
    /// Assets.
    pub assets: Vec<SubAccountFuturesBalance>,
    /// Can deposit?
    pub can_deposit: bool,
    /// Can trade?
    pub can_trade: bool,
    /// Can withdraw?
    pub can_withdraw: bool,
    /// Fee Tier.
    pub fee_tier: usize,
    /// Max withdraw amount.
    pub max_withdraw_amount: Option<Decimal>,
    /// Total initial margin.
    pub total_initial_margin: Option<Decimal>,
    /// Total maintenance margin.
    pub total_maintenance_margin: Option<Decimal>,
    /// Total margin balance.
    pub total_margin_balance: Option<Decimal>,
    /// Total open order initial margin.
    pub total_open_order_initial_margin: Option<Decimal>,
    /// Total position initial margin.
    pub total_position_initial_margin: Option<Decimal>,
    /// Total unrealized profit.
    pub total_unrealized_profit: Option<Decimal>,
    /// Total wallet balance.
    pub total_wallet_balance: Option<Decimal>,
    /// Update time.
    pub update_time: i64,
}

/// Details of sub-account in futures account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubAccountFutures {
    /// USD-Margin Futures.
    #[serde(rename = "futureAccountResp")]
    Usd(SubAccountFuturesInner),
    /// Coin-Margin Futures.
    #[serde(rename = "deliveryAccountResp")]
    Coin(SubAccountFuturesInner),
}

impl TryFrom<Data> for SubAccountFutures {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::SubAccountFutures(data) => Ok(data),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
