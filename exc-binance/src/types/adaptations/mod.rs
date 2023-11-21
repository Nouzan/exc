use std::ops::Bound;

use exc_core::ExchangeError;
use time::OffsetDateTime;

mod book;
mod candle;
mod instrument;
mod ticker;
mod trade;
mod trading;
mod utils;

pub(crate) fn from_timestamp(ts: i64) -> Result<OffsetDateTime, ExchangeError> {
    OffsetDateTime::from_unix_timestamp_nanos((ts as i128) * 1_000_000)
        .map_err(|err| ExchangeError::Other(anyhow!("parse timestamp error: {err}")))
}

pub(crate) fn to_timestamp(ts: &OffsetDateTime) -> Result<i64, ExchangeError> {
    let ts = ts.unix_timestamp_nanos() / 1_000_000;
    if ts > (i64::MAX as i128) {
        return Err(ExchangeError::Other(anyhow!("datetime too big: {ts}")));
    }
    Ok(ts as i64)
}

pub(crate) fn start_bound_to_timestamp(
    start: Bound<&OffsetDateTime>,
) -> Result<Option<i64>, ExchangeError> {
    match start {
        Bound::Included(ts) => Ok(Some(to_timestamp(ts)?)),
        Bound::Excluded(ts) => Ok(Some(to_timestamp(&(*ts + time::Duration::SECOND))?)),
        Bound::Unbounded => Ok(None),
    }
}

pub(crate) fn end_bound_to_timestamp(
    end: Bound<&OffsetDateTime>,
) -> Result<Option<i64>, ExchangeError> {
    match end {
        Bound::Included(ts) => Ok(Some(to_timestamp(ts)?)),
        Bound::Excluded(ts) => Ok(Some(to_timestamp(&(*ts - time::Duration::SECOND))?)),
        Bound::Unbounded => Ok(None),
    }
}
