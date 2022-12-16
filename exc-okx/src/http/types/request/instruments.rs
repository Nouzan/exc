use serde::Serialize;

/// History Candles.
#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Instruments {
    /// Instrument type.
    pub(crate) inst_type: String,
    /// Underlying.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) uly: Option<String>,
    /// Instrument family.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inst_family: Option<String>,
    /// Instrument id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inst_id: Option<String>,
}

impl Instruments {
    /// Spot.
    pub fn spot() -> Self {
        Self {
            inst_type: "SPOT".to_string(),
            ..Default::default()
        }
    }

    /// Margin.
    pub fn margin() -> Self {
        Self {
            inst_type: "MARGIN".to_string(),
            ..Default::default()
        }
    }

    /// Swap.
    pub fn swap() -> Self {
        Self {
            inst_type: "SWAP".to_string(),
            ..Default::default()
        }
    }

    /// Futures.
    pub fn futures() -> Self {
        Self {
            inst_type: "FUTURES".to_string(),
            ..Default::default()
        }
    }

    /// Options.
    pub fn options(underlying: &str) -> Self {
        Self {
            inst_type: "OPTION".to_string(),
            uly: Some(underlying.to_string()),
            ..Default::default()
        }
    }

    /// With instrument id.
    pub fn with_inst(mut self, inst: &str) -> Self {
        self.inst_id = Some(inst.to_string());
        self
    }
}
