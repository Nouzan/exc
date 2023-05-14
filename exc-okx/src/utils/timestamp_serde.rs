use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use std::{fmt, str::FromStr};
use thiserror::Error;
use time::OffsetDateTime;

pub use time::serde::timestamp::serialize;

/// Deserialize an `OffsetDateTime` from its Unix timestamp
pub fn deserialize<'a, D: Deserializer<'a>>(deserializer: D) -> Result<OffsetDateTime, D::Error> {
    let s = <String>::deserialize(deserializer)?;
    let ts: i64 = s.parse().map_err(|_| {
        <D::Error>::invalid_value(
            Unexpected::Str(s.as_str()),
            &"a str contains valid timestamp",
        )
    })?;
    OffsetDateTime::from_unix_timestamp_nanos((ts as i128) * 1_000_000)
        .map_err(|err| <D::Error>::invalid_value(Unexpected::Signed(ts), &err))
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

/// Okex timestamp.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Timestamp(pub OffsetDateTime);

impl FromStr for Timestamp {
    type Err = DeserializeTimestampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ts: i64 = s.parse()?;
        let ts = OffsetDateTime::from_unix_timestamp_nanos((ts as i128) * 1_000_000)?;
        Ok(Self(ts))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ts = self.0.unix_timestamp_nanos() / 1_000_000;
        write!(f, "{ts}")
    }
}
