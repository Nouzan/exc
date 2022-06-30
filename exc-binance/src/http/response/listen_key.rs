use std::fmt;

use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

/// Listen key.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
    listen_key: String,
}

impl ListenKey {
    /// Convert to a [`&str`].
    pub fn as_str(&self) -> &str {
        self.listen_key.as_str()
    }
}

impl fmt::Display for ListenKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.listen_key)
    }
}

impl TryFrom<Data> for ListenKey {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::ListenKey(key) => Ok(key),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
