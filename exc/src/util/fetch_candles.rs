use std::ops::{Bound, RangeBounds};

use crate::core::types::Period;
use futures::{future::BoxFuture, FutureExt};
use time::OffsetDateTime;
use tower::ServiceExt;

use crate::core::types::candle::{CandleStream, QueryCandles};

use crate::ExcService;

/// Fetch candles service.
pub trait FetchCandlesService {
    /// Query candles
    fn fetch_candles(
        &mut self,
        inst: &str,
        period: Period,
        start: Bound<OffsetDateTime>,
        end: Bound<OffsetDateTime>,
    ) -> BoxFuture<'_, crate::Result<CandleStream>>;
}

impl<S> FetchCandlesService for S
where
    S: ExcService<QueryCandles> + Send,
    S::Future: Send,
{
    fn fetch_candles(
        &mut self,
        inst: &str,
        period: Period,
        start: Bound<OffsetDateTime>,
        end: Bound<OffsetDateTime>,
    ) -> BoxFuture<'_, crate::Result<CandleStream>> {
        ServiceExt::<QueryCandles>::oneshot(
            self.as_service(),
            QueryCandles::new(inst, period, (start, end)),
        )
        .boxed()
    }
}

/// Helper methods for fetch candle services.
pub trait FetchCandlesServiceExt: FetchCandlesService {
    /// Fetch candles in range.
    fn fetch_candles_range(
        &mut self,
        inst: impl AsRef<str>,
        period: Period,
        range: impl RangeBounds<OffsetDateTime>,
    ) -> BoxFuture<'_, crate::Result<CandleStream>> {
        self.fetch_candles(
            inst.as_ref(),
            period,
            range.start_bound().cloned(),
            range.end_bound().cloned(),
        )
    }
}

impl<S: FetchCandlesService> FetchCandlesServiceExt for S {}

impl<'a> FetchCandlesService for Box<dyn FetchCandlesService + 'a> {
    fn fetch_candles(
        &mut self,
        inst: &str,
        period: Period,
        start: Bound<OffsetDateTime>,
        end: Bound<OffsetDateTime>,
    ) -> BoxFuture<'_, crate::Result<CandleStream>> {
        self.as_mut().fetch_candles(inst, period, start, end)
    }
}

#[cfg(feature = "fetch-candles")]
pub use crate::core::util::fetch_candles::*;
