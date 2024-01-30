use exc_core::Str;
use serde::Deserialize;

use crate::error::OkxError;

/// Instrument Tag Params.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    /// Instrument family.
    #[serde(alias = "family")]
    pub inst_family: Option<Str>,
    /// Instrument underlying.
    pub uly: Option<Str>,
}

/// Parse Instrument Tag.
pub fn parse_inst_tag(tag: &str) -> Result<(Str, Params), OkxError> {
    let (tag, params) = tag
        .split_once('?')
        .map(|(ty, params)| serde_qs::from_str::<Params>(params).map(|p| (ty, p)))
        .transpose()
        .map_err(|err| OkxError::UnexpectedDataType(err.into()))?
        .unwrap_or_else(|| (tag, Params::default()));
    Ok((Str::new(tag), params))
}
