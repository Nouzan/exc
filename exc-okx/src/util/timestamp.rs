use std::ops::Bound;

use time::OffsetDateTime;

/// Timestamp to millis.
pub fn ts_to_millis(ts: &OffsetDateTime) -> Option<u64> {
    let nanos = ts.unix_timestamp_nanos();
    if nanos < 0 {
        None
    } else {
        Some((nanos / 1_000_000) as u64)
    }
}

/// Millis to timestamp.
pub fn millis_to_ts(millis: u64) -> Option<OffsetDateTime> {
    let nanos = (millis as i128) * 1_000_000;
    OffsetDateTime::from_unix_timestamp_nanos(nanos).ok()
}

/// Start bound to millis.
pub fn start_bound_to_millis(bound: Bound<&OffsetDateTime>) -> Option<u64> {
    match bound {
        Bound::Unbounded => None,
        Bound::Excluded(ts) => ts_to_millis(ts),
        Bound::Included(ts) => {
            ts_to_millis(ts).and_then(|x| if x > 0 { Some(x - 1) } else { None })
        }
    }
}

/// End bound to millis.
pub fn end_bound_to_millis(bound: Bound<&OffsetDateTime>) -> Option<u64> {
    match bound {
        Bound::Unbounded => None,
        Bound::Excluded(ts) => ts_to_millis(ts),
        Bound::Included(ts) => ts_to_millis(ts).map(|x| x + 1),
    }
}
