use futures::stream::BoxStream;
use indicator::Period;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::ops::{Bound, RangeBounds};
use time::OffsetDateTime;

use crate::ExchangeError;

use super::Request;

/// Candle Stream.
pub type CandleStream = BoxStream<'static, Result<Candle, ExchangeError>>;

/// Query candles.
#[derive(Debug)]
pub struct QueryCandles {
    inst: String,
    period: Period,
    start: Bound<OffsetDateTime>,
    end: Bound<OffsetDateTime>,
}

impl QueryCandles {
    /// Create a new query.
    pub fn new<R>(inst: &str, period: Period, range: R) -> Self
    where
        R: RangeBounds<OffsetDateTime>,
    {
        let inst = inst.to_string();
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
        Self {
            inst,
            period,
            start,
            end,
        }
    }

    /// Get Instrument.
    pub fn inst(&self) -> &str {
        self.inst.as_str()
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

impl Request for QueryCandles {
    type Response = CandleStream;
}

/// Query last `n` candles in range.
#[derive(Debug)]
pub struct QueryLastCandles {
    query: QueryCandles,
    last: usize,
}

impl QueryLastCandles {
    /// Create a new query.
    pub fn new<R>(inst: &str, period: Period, range: R, last: usize) -> Self
    where
        R: RangeBounds<OffsetDateTime>,
    {
        let query = QueryCandles::new(inst, period, range);
        Self { query, last }
    }

    /// Get last.
    pub fn last(&self) -> usize {
        self.last
    }

    /// Get query.
    pub fn query(&self) -> &QueryCandles {
        &self.query
    }
}

impl Request for QueryLastCandles {
    type Response = CandleStream;
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
