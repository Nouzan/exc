use serde::Deserialize;

use crate::http::error::RestError;

use super::Data;

/// Error message.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMessage {
    /// Code.
    pub code: i64,
    /// Message.
    #[serde(rename = "msg")]
    pub message: String,
}

impl TryFrom<Data> for ErrorMessage {
    type Error = RestError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Error(e) => Err(RestError::Api(e.code, e.message)),
            _ => Err(RestError::UnexpectedResponseType(anyhow::anyhow!(
                "{value:?}"
            ))),
        }
    }
}
