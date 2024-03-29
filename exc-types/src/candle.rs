use derive_more::Display;
use futures::{stream::BoxStream, Stream, StreamExt};
pub use indicator::{window::mode::tumbling::period::PeriodKind, Period};
use indicator::{Tick, TickValue, Tickable};
use positions::prelude::Str;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Bound, RangeBounds},
};
use time::OffsetDateTime;

use exc_service::{ExchangeError, Request};

/// Candle Stream.
pub struct CandleStream {
    forward: bool,
    stream: BoxStream<'static, Result<Candle, ExchangeError>>,
}

impl CandleStream {
    /// Create a new candle stream that produce candles forward.
    pub fn new_forward(
        stream: impl Stream<Item = Result<Candle, ExchangeError>> + Send + 'static,
    ) -> Self {
        Self {
            forward: true,
            stream: stream.boxed(),
        }
    }

    /// Create a new candle stream that produce candles backward.
    pub fn new_backward(
        stream: impl Stream<Item = Result<Candle, ExchangeError>> + Send + 'static,
    ) -> Self {
        Self {
            forward: false,
            stream: stream.boxed(),
        }
    }

    /// Is forward.
    pub fn is_forward(&self) -> bool {
        self.forward
    }
}

impl Stream for CandleStream {
    type Item = Result<Candle, ExchangeError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

/// Query candles.
#[derive(Debug, Clone)]
pub struct QueryCandles {
    /// Instrument.
    pub inst: Str,
    /// Period.
    pub period: Period,
    /// Start.
    pub start: Bound<OffsetDateTime>,
    /// End.
    pub end: Bound<OffsetDateTime>,
}

fn fmt_ts_start_bound(bound: &Bound<OffsetDateTime>) -> String {
    match bound {
        Bound::Unbounded => "(".to_string(),
        Bound::Excluded(ts) => format!("({ts}"),
        Bound::Included(ts) => format!("[{ts}"),
    }
}

fn fmt_ts_end_bound(bound: &Bound<OffsetDateTime>) -> String {
    match bound {
        Bound::Unbounded => ")".to_string(),
        Bound::Excluded(ts) => format!("{ts})"),
        Bound::Included(ts) => format!("{ts}]"),
    }
}

impl fmt::Display for QueryCandles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}, {}",
            self.inst,
            self.period,
            fmt_ts_start_bound(&self.start),
            fmt_ts_end_bound(&self.end)
        )
    }
}

impl QueryCandles {
    /// Create a new query.
    pub fn new<R>(inst: impl AsRef<str>, period: Period, range: R) -> Self
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
        Self {
            inst: Str::new(inst),
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

    /// Is empty.
    pub fn is_empty(&self) -> bool {
        match (self.start_bound(), self.end_bound()) {
            (Bound::Unbounded, _) => false,
            (_, Bound::Unbounded) => false,
            (Bound::Included(start), Bound::Included(end)) => *start > *end,
            (Bound::Included(start), Bound::Excluded(end)) => *start >= *end,
            (Bound::Excluded(start), Bound::Included(end)) => *start >= *end,
            (Bound::Excluded(start), Bound::Excluded(end)) => *start >= *end,
        }
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
/// Return a candle stream that produce the last `last` candles backward.
#[derive(Debug, Clone)]
pub struct QueryLastCandles {
    /// Query.
    pub query: QueryCandles,
    /// Last.
    pub last: usize,
}

impl fmt::Display for QueryLastCandles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-(-{})", self.query, self.last)
    }
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

/// Query first `n` candles in range.
/// Return a candle stream that produce the first `fisrt` candles forward.
#[derive(Debug, Clone)]
pub struct QueryFirstCandles {
    /// Query.
    pub query: QueryCandles,
    /// First.
    pub first: usize,
}

impl fmt::Display for QueryFirstCandles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-({})", self.query, self.first)
    }
}

impl QueryFirstCandles {
    /// Create a new query.
    pub fn new<R>(inst: &str, period: Period, range: R, first: usize) -> Self
    where
        R: RangeBounds<OffsetDateTime>,
    {
        let query = QueryCandles::new(inst, period, range);
        Self { query, first }
    }

    /// Get first.
    pub fn first(&self) -> usize {
        self.first
    }

    /// Get query.
    pub fn query(&self) -> &QueryCandles {
        &self.query
    }
}

impl Request for QueryFirstCandles {
    type Response = CandleStream;
}

/// Candle (OHLCV).
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[display(fmt = "ts={ts} ohlcv=[{open}, {high}, {low}, {close}, {volume}]")]
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

impl Tickable for Candle {
    type Value = Self;

    fn tick(&self) -> Tick {
        Tick::new(self.ts)
    }

    fn value(&self) -> &Self::Value {
        self
    }

    fn into_tick_value(self) -> TickValue<Self::Value> {
        TickValue::new(self.ts, self)
    }
}
