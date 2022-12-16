use exc_core::types::instrument::InstrumentMeta;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};
use std::collections::HashSet;
use time::OffsetDateTime;

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
    pub inst_id: String,

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
    pub ct_val_ccy: String,
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

    /// Underlying, e.g. `BTC-USD`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub uly: String,

    /// Settlement and margin currency, e.g. `BTC`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub settle_ccy: String,

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
    pub base_ccy: String,

    /// Quote currency, e.g. `USDT` in `BTC-USDT`.
    /// Only applicable to `SPOT`.
    pub quote_ccy: String,

    /// Leverage
    /// Not applicable to `SPOT`, used to distinguish between `MARGIN` and `SPOT`.
    #[serde_as(as = "NoneAsEmptyString")]
    pub lever: Option<Decimal>,
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

    /// Underlying, e.g. `BTC-USD`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub uly: String,

    /// Settlement and margin currency, e.g. `BTC`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub settle_ccy: String,

    /// Listing time.
    /// Only applicable to `FUTURES`/`SWAP`/`OPTION`.
    #[serde(with = "crate::utils::timestamp_serde_option")]
    pub list_time: Option<OffsetDateTime>,

    /// Leverage
    /// Not applicable to `SPOT`, used to distinguish between `MARGIN` and `SPOT`.
    pub lever: Decimal,

    /// Contract type, `linear`: linear contract `inverse`: inverse contract.
    /// Applicable to `SWAP` and `Futures`.
    pub ct_type: String,
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

    /// Underlying, e.g. `BTC-USD`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub uly: String,

    /// Settlement and margin currency, e.g. `BTC`.
    /// Only applicable to `FUTURES/SWAP/OPTION`.
    pub settle_ccy: String,

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
    pub ct_type: String,
}

impl From<OkxInstrumentMeta> for InstrumentMeta<Decimal> {
    fn from(meta: OkxInstrumentMeta) -> Self {
        let is_reversed = meta
            .common()
            .inst_id
            .split('-')
            .collect::<HashSet<_>>()
            .contains("USD");
        let unit = if let Some(contract) = meta.as_contract() {
            contract.ct_val
        } else {
            Decimal::ONE
        };
        let price_tick = meta.common().tick_sz;
        let size_tick = meta.common().lot_sz;
        let min_size = meta.common().min_sz;
        InstrumentMeta {
            name: meta.into_common().inst_id,
            is_reversed,
            unit,
            price_tick,
            size_tick,
            min_size,
            min_value: Decimal::ZERO,
        }
    }
}
