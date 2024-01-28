use exc_core::{
    symbol::ExcSymbol,
    types::instrument::{Attributes, InstrumentMeta},
    Asset, ParseAssetError, Str,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};
use time::OffsetDateTime;

use crate::error::OkxError;

/// Okx Instrument Meta.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "instType", rename_all = "UPPERCASE")]
pub enum OkxInstrumentMeta {
    /// Spot.
    Spot(SpotMeta),

    /// Margin.
    Margin(SpotMeta),

    /// Swap.
    Swap(SwapMeta),

    /// Futures.
    Futures(FuturesMeta),

    /// Option.
    Option(OptionMeta),
}

impl OkxInstrumentMeta {
    /// Common.
    pub fn common(&self) -> &CommonMeta {
        match self {
            Self::Spot(SpotMeta { common, .. })
            | Self::Margin(SpotMeta { common, .. })
            | Self::Swap(SwapMeta { common, .. })
            | Self::Futures(FuturesMeta { common, .. })
            | Self::Option(OptionMeta { common, .. }) => common,
        }
    }

    /// Into Common.
    pub fn into_common(self) -> CommonMeta {
        match self {
            Self::Spot(SpotMeta { common, .. })
            | Self::Margin(SpotMeta { common, .. })
            | Self::Swap(SwapMeta { common, .. })
            | Self::Futures(FuturesMeta { common, .. })
            | Self::Option(OptionMeta { common, .. }) => common,
        }
    }

    /// Expire Time.
    pub fn expire_time(&self) -> Option<OffsetDateTime> {
        match self {
            Self::Futures(FuturesMeta { exp_time, .. })
            | Self::Option(OptionMeta { exp_time, .. }) => *exp_time,
            _ => None,
        }
    }

    /// Convert to common meta.
    pub fn as_contract(&self) -> Option<&ContractCommonMeta> {
        match self {
            Self::Futures(FuturesMeta { contract, .. })
            | Self::Swap(SwapMeta { contract, .. })
            | Self::Option(OptionMeta { contract, .. }) => Some(contract),
            _ => None,
        }
    }

    /// Is reversed.
    pub fn is_reversed(&self) -> bool {
        match self {
            Self::Spot(_) => false,
            Self::Margin(_) => false,
            Self::Futures(FuturesMeta { contract, .. }) => contract.ct_val_ccy == "USD",
            Self::Swap(SwapMeta { contract, .. }) => contract.ct_val_ccy == "USD",
            Self::Option(_) => false,
        }
    }

    /// Convert to an [`ExcSymbol`].
    pub fn to_exc_symbol(&self) -> Result<ExcSymbol, OkxError> {
        match self {
            Self::Spot(meta) => Some(ExcSymbol::spot(&meta.base_ccy, &meta.quote_ccy)),
            Self::Margin(meta) => Some(ExcSymbol::margin(&meta.base_ccy, &meta.quote_ccy)),
            Self::Futures(FuturesMeta {
                contract, common, ..
            }) => common.inst_id.split('-').nth(2).and_then(|date| {
                ExcSymbol::futures_with_str(&contract.ct_val_ccy, &contract.settle_ccy, date)
            }),
            Self::Swap(SwapMeta { contract, .. }) => Some(ExcSymbol::perpetual(
                &contract.ct_val_ccy,
                &contract.settle_ccy,
            )),
            Self::Option(OptionMeta { common, .. }) => {
                let mut parts = common.inst_id.split('-');
                let base = parts
                    .next()
                    .map(|s| s.parse())
                    .transpose()
                    .map_err(|err: ParseAssetError| OkxError::ParseSymbol(err.into()))?;
                let quote = parts
                    .next()
                    .map(|s| s.parse())
                    .transpose()
                    .map_err(|err: ParseAssetError| OkxError::ParseSymbol(err.into()))?;
                let date = parts.next();
                let price = parts.next();
                parts.next().and_then(|ty| match ty {
                    "c" | "C" => ExcSymbol::call_with_str(&base?, &quote?, date?, price?),
                    "p" | "P" => ExcSymbol::put_with_str(&base?, &quote?, date?, price?),
                    _ => None,
                })
            }
        }
        .ok_or(OkxError::FailedToBuildExcSymbol)
    }
}

/// Instrument State.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InstrumentState {
    /// Live.
    Live,
    /// Suspend.
    Suspend,
    /// Expired.
    Expired,
    /// Preopen.
    Preopen,
    /// Test.
    Test,
}

/// Common Meta.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonMeta {
    /// Instrument ID.
    pub inst_id: Str,

    /// Fee Schedule.
    #[serde_as(as = "DisplayFromStr")]
    pub category: usize,

    /// Tick size, e.g. `0.0001`.
    pub tick_sz: Decimal,

    /// Lot size, e.g. `BTC-USDT-SWAP`: `1`
    pub lot_sz: Decimal,

    /// Minimum order size
    pub min_sz: Decimal,

    /// Instrument status.
    /// `live`
    /// `suspend`
    /// `expired`
    /// `preopen`
    pub state: InstrumentState,
}

/// Contract Common Meta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractCommonMeta {
    /// Contract value.
    pub ct_val: Decimal,

    /// Contract multiplier.
    pub ct_mult: Decimal,

    /// Contract value currency.
    pub ct_val_ccy: Asset,

    /// Underlying, e.g. `BTC-USD`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub uly: Str,

    /// Settlement and margin currency, e.g. `BTC`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub settle_ccy: Asset,
}

/// Option Type.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OptionType {
    /// Put.
    #[serde(rename = "P")]
    Put,
    /// Call.
    #[serde(rename = "C")]
    Call,
}

/// Option Meta.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionMeta {
    /// Common meta.
    #[serde(flatten)]
    pub common: CommonMeta,

    /// Contract Common meta.
    #[serde(flatten)]
    pub contract: ContractCommonMeta,

    /// Option type, `C`: Call `P`: Put
    /// Only applicable to `OPTION`.
    pub opt_type: OptionType,

    /// Strike price.
    /// Only applicable to `OPTION`.
    pub stk: Decimal,

    /// Listing time.
    /// Only applicable to `FUTURES`/`SWAP`/`OPTION`.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    pub list_time: Option<OffsetDateTime>,

    /// Expiry time.
    /// Only applicable to `FUTURES`/`OPTION`.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    pub exp_time: Option<OffsetDateTime>,
}

/// Spot Meta.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotMeta {
    /// Common meta.
    #[serde(flatten)]
    pub common: CommonMeta,

    /// Base currency, e.g. `BTC` in `BTC-USDT`.
    /// Only applicable to `SPOT`.
    pub base_ccy: Asset,

    /// Quote currency, e.g. `USDT` in `BTC-USDT`.
    /// Only applicable to `SPOT`.
    pub quote_ccy: Asset,

    /// Leverage
    /// Not applicable to `SPOT`, used to distinguish between `MARGIN` and `SPOT`.
    #[serde_as(as = "NoneAsEmptyString")]
    pub lever: Option<Decimal>,
}

/// Contract Type.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ContractType {
    /// Linear.
    #[serde(rename = "linear")]
    Linear,
    /// Inverse.
    #[serde(rename = "inverse")]
    Inverse,
}

/// Swap Meta.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapMeta {
    /// Common meta.
    #[serde(flatten)]
    pub common: CommonMeta,

    /// Contract Common meta.
    #[serde(flatten)]
    pub contract: ContractCommonMeta,

    /// Listing time.
    /// Only applicable to `FUTURES`/`SWAP`/`OPTION`.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    pub list_time: Option<OffsetDateTime>,

    /// Leverage
    /// Not applicable to `SPOT`, used to distinguish between `MARGIN` and `SPOT`.
    pub lever: Decimal,

    /// Contract type, `linear`: linear contract `inverse`: inverse contract.
    /// Applicable to `SWAP` and `Futures`.
    pub ct_type: ContractType,
}

/// Futures Meta.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesMeta {
    /// Common meta.
    #[serde(flatten)]
    pub common: CommonMeta,

    /// Contract Common meta.
    #[serde(flatten)]
    pub contract: ContractCommonMeta,

    /// Listing time.
    /// Only applicable to `FUTURES`/`SWAP`/`OPTION`.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    pub list_time: Option<OffsetDateTime>,

    /// Expiry time.
    /// Only applicable to `FUTURES`/`OPTION`.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    pub exp_time: Option<OffsetDateTime>,

    /// Leverage
    /// Not applicable to `SPOT`, used to distinguish between `MARGIN` and `SPOT`.
    pub lever: Decimal,

    /// Alias.
    /// `this_week`
    /// `next_week`
    /// `quarter`
    /// `next_quarter`
    /// Only applicable to `FUTURES`.
    pub alias: String,

    /// Contract type, `linear`: linear contract `inverse`: inverse contract.
    /// Applicable to `SWAP` and `Futures`.
    pub ct_type: ContractType,
}

impl TryFrom<OkxInstrumentMeta> for InstrumentMeta<Decimal> {
    type Error = OkxError;

    fn try_from(meta: OkxInstrumentMeta) -> Result<Self, Self::Error> {
        let unit = if let Some(contract) = meta.as_contract() {
            contract.ct_val * contract.ct_mult
        } else {
            Decimal::ONE
        };
        let common = meta.common();
        let attrs = Attributes {
            reversed: meta.is_reversed(),
            unit,
            price_tick: common.tick_sz,
            size_tick: common.lot_sz,
            min_size: common.min_sz,
            min_value: Decimal::ZERO,
        };
        Ok(
            InstrumentMeta::new(common.inst_id.as_str(), meta.to_exc_symbol()?, attrs)
                .with_live(matches!(common.state, InstrumentState::Live)),
        )
    }
}
