use std::ops::RangeBounds;

use indicator::Period;
use time::OffsetDateTime;
use tower::{util::Oneshot, ServiceExt};

use crate::types::candle::QueryCandles;

use super::ExchangeService;

/// Fetch candles service.
pub trait FetchCandlesService: ExchangeService<QueryCandles> {
    /// Query candles
    fn fetch_candles<R>(
        &mut self,
        inst: &str,
        period: Period,
        range: R,
    ) -> Oneshot<&mut Self, QueryCandles>
    where
        Self: Sized,
        R: RangeBounds<OffsetDateTime>,
    {
        ServiceExt::<QueryCandles>::oneshot(self, QueryCandles::new(inst, period, range))
    }
}

impl<S> FetchCandlesService for S where S: ExchangeService<QueryCandles> {}
