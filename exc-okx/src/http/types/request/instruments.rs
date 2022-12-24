use exc_core::Str;
use serde::Serialize;

/// History Candles.
#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Instruments {
    /// Instrument type.
    pub(crate) inst_type: Str,
    /// Underlying.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) uly: Option<Str>,
    /// Instrument family.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inst_family: Option<Str>,
    /// Instrument id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inst_id: Option<Str>,
}

impl Instruments {
    /// Spot.
    pub fn spot() -> Self {
        Self {
            inst_type: Str::new_inline("SPOT"),
            ..Default::default()
        }
    }

    /// Margin.
    pub fn margin() -> Self {
        Self {
            inst_type: Str::new_inline("MARGIN"),
            ..Default::default()
        }
    }

    /// Swap.
    pub fn swap() -> Self {
        Self {
            inst_type: Str::new_inline("SWAP"),
            ..Default::default()
        }
    }

    /// Futures.
    pub fn futures() -> Self {
        Self {
            inst_type: Str::new_inline("FUTURES"),
            ..Default::default()
        }
    }

    /// Options.
    pub fn options(underlying: &str) -> Self {
        Self {
            inst_type: Str::new_inline("OPTION"),
            uly: Some(Str::new(underlying)),
            ..Default::default()
        }
    }

    /// With instrument id.
    pub fn with_inst(mut self, inst: &str) -> Self {
        self.inst_id = Some(Str::new(inst));
        self
    }
}
