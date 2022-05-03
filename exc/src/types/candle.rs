use indicator::Period;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::ops::{Bound, RangeBounds};
use time::OffsetDateTime;

/// Query candles.
#[derive(Debug)]
pub struct QueryCandles {
    period: Period,
    start: Bound<OffsetDateTime>,
    end: Bound<OffsetDateTime>,
}

impl QueryCandles {
    /// Create a new query.
    pub fn new<R>(period: Period, range: R) -> Self
    where
        R: RangeBounds<OffsetDateTime>,
    {
        let offset = period.utc_offset();
        let start = match range.start_bound() {
            Bound::Unbounded => Bound::Unbounded,
            Bound::Included(&t) => Bound::Included(t.to_offset(offset)),
            Bound::Excluded(&t) => Bound::Excluded(t.to_offset(offset)),
        };
        let end = match range.end_bound() {
            Bound::Unbounded => Bound::Unbounded,
            Bound::Included(&t) => Bound::Included(t.to_offset(offset)),
            Bound::Excluded(&t) => Bound::Excluded(t.to_offset(offset)),
        };
        Self { period, start, end }
    }

    /// Get period.
    pub fn period(&self) -> Period {
        self.period
    }
}

impl RangeBounds<OffsetDateTime> for QueryCandles {
    fn start_bound(&self) -> Bound<&OffsetDateTime> {
        match &self.start {
            Bound::Unbounded => Bound::Unbounded,
            Bound::Included(t) => Bound::Included(t),
            Bound::Excluded(t) => Bound::Excluded(t),
        }
    }

    fn end_bound(&self) -> Bound<&OffsetDateTime> {
        match &self.end {
            Bound::Unbounded => Bound::Unbounded,
            Bound::Included(t) => Bound::Included(t),
            Bound::Excluded(t) => Bound::Excluded(t),
        }
    }
}

/// Query last `n` candles in range.
#[derive(Debug)]
pub struct QueryLastCandles {
    query: QueryCandles,
    last: usize,
}

impl QueryLastCandles {
    /// Create a new query.
    pub fn new<R>(period: Period, range: R, last: usize) -> Self
    where
        R: RangeBounds<OffsetDateTime>,
    {
        let query = QueryCandles::new(period, range);
        Self { query, last }
    }

    /// Get last.
    pub fn last(&self) -> usize {
        self.last
    }
}

impl RangeBounds<OffsetDateTime> for QueryLastCandles {
    fn start_bound(&self) -> Bound<&OffsetDateTime> {
        self.query.start_bound()
    }

    fn end_bound(&self) -> Bound<&OffsetDateTime> {
        self.query.end_bound()
    }
}

/// Candle (OHLCV).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub ts: OffsetDateTime,
    /// The open price.
    pub open: Decimal,
    /// The highest price.
    pub high: Decimal,
    /// The lowest price.
    pub low: Decimal,
    /// The last price.
    pub close: Decimal,
    /// The volume.
    #[serde(default)]
    pub volume: Decimal,
}
