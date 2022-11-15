use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer,
};
use thiserror::Error;
use time::OffsetDateTime;

pub use time::serde::timestamp::option::serialize;

/// Deserialize an `OffsetDateTime` from its Unix timestamp
pub fn deserialize<'a, D: Deserializer<'a>>(
    deserializer: D,
) -> Result<Option<OffsetDateTime>, D::Error> {
    let s = <String>::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    let ts: i64 = s.parse().map_err(|_| {
        <D::Error>::invalid_value(
            Unexpected::Str(s.as_str()),
            &"a str contains valid timestamp",
        )
    })?;
    OffsetDateTime::from_unix_timestamp_nanos((ts as i128) * 1_000_000)
        .map_err(|err| <D::Error>::invalid_value(Unexpected::Signed(ts), &err))
        .map(Some)
}

/// Deserialize timestamp error.
#[derive(Debug, Error)]
pub enum DeserializeTimestampError {
    /// Parse `str` to `i64` error.
    #[error(transparent)]
    ParseI64Error(#[from] std::num::ParseIntError),

    /// Convert into [`OffsetDateTime`] error.
    #[error(transparent)]
    OffsetDateTime(#[from] time::error::ComponentRange),
}
