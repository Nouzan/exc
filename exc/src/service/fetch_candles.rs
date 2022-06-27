use std::ops::{Bound, RangeBounds};

use async_stream::try_stream;
use futures::{future::BoxFuture, FutureExt, StreamExt};
use indicator::Period;
use time::OffsetDateTime;
use tower::{util::Oneshot, Layer, Service, ServiceExt};

use crate::{
    types::{
        candle::{CandleStream, QueryCandles, QueryLastCandles},
        QueryFirstCandles,
    },
    ExchangeError,
};

use super::{Exc, ExcMut, ExchangeService};

/// Fetch candles service.
pub trait FetchCandlesService: ExchangeService<QueryCandles> {
    /// Query candles
    fn fetch_candles<R>(
        &mut self,
        inst: &str,
        period: Period,
        range: R,
    ) -> Oneshot<ExcMut<'_, Self>, QueryCandles>
    where
        Self: Sized,
        R: RangeBounds<OffsetDateTime>,
    {
        ServiceExt::<QueryCandles>::oneshot(
            self.as_service_mut(),
            QueryCandles::new(inst, period, range),
        )
    }
}

impl<S> FetchCandlesService for S where S: ExchangeService<QueryCandles> {}

use std::num::NonZeroUsize;
use tower::buffer::Buffer;

const DEFAULT_BOUND: usize = 64;

/// Fetch candles backward layer.
pub struct FetchCandlesBackwardLayer {
    bound: usize,
    limit: NonZeroUsize,
}

impl FetchCandlesBackwardLayer {
    /// Create a new fetch candles backward layer.
    /// # Panic
    /// Panic if `limit` is zero.
    pub fn new(limit: usize, bound: usize) -> Self {
        Self {
            bound: bound + 1,
            limit: NonZeroUsize::new(limit).unwrap(),
        }
    }

    /// Create a new fetch candles backward layer, with default buffer bound.
    /// # Panic
    /// Panic if `limit` is zero.
    pub fn with_default_bound(limit: usize) -> Self {
        Self::new(limit, DEFAULT_BOUND)
    }
}

impl<S> Layer<S> for FetchCandlesBackwardLayer
where
    S: ExchangeService<QueryLastCandles> + Send + 'static,
    S::Future: Send,
{
    type Service = FetchCandlesBackward<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FetchCandlesBackward {
            svc: Buffer::new(inner.into_service(), self.bound),
            limit: self.limit,
        }
    }
}

/// Fetch candles backward.
pub struct FetchCandlesBackward<S>
where
    S: ExchangeService<QueryLastCandles> + 'static,
{
    svc: Buffer<Exc<S>, QueryLastCandles>,
    limit: NonZeroUsize,
}

impl<S> Service<QueryCandles> for FetchCandlesBackward<S>
where
    S: ExchangeService<QueryLastCandles> + 'static,
    S::Future: Send,
{
    type Response = CandleStream;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.svc, cx).map_err(ExchangeError::from)
    }

    fn call(&mut self, query: QueryCandles) -> Self::Future {
        let mut query = QueryLastCandles {
            query,
            last: self.limit.get(),
        };
        let mut svc = self.svc.clone();
        async move {
	    let stream = try_stream!{
		loop {
            tracing::trace!("query candles: {query}");
            if query.query().is_empty() { break; }
		    let mut stream = (&mut svc).oneshot(query.clone()).await.map_err(ExchangeError::Layer)?;
		    let mut next = None;
		    while let Some(c) = stream.next().await {
			let c = c?;
			next = Some(c.ts);
			yield c;
		    }
		    if let Some(next) = next {
			query.query.end = Bound::Excluded(next);
		    } else {
			break;
		    }
		}
	    };
	    Ok(stream.boxed())
	}.boxed()
    }
}

/// Fetch candles forward layer.
pub struct FetchCandlesForwardLayer {
    bound: usize,
    limit: NonZeroUsize,
}

impl FetchCandlesForwardLayer {
    /// Create a new fetch candles forward layer.
    /// # Panic
    /// Panic if `limit` is zero.
    pub fn new(limit: usize, bound: usize) -> Self {
        Self {
            bound: bound + 1,
            limit: NonZeroUsize::new(limit).unwrap(),
        }
    }

    /// Create a new fetch candles forward layer, with default buffer bound.
    /// # Panic
    /// Panic if `limit` is zero.
    pub fn with_default_bound(limit: usize) -> Self {
        Self::new(limit, DEFAULT_BOUND)
    }
}

impl<S> Layer<S> for FetchCandlesForwardLayer
where
    S: ExchangeService<QueryFirstCandles> + Send + 'static,
    S::Future: Send,
{
    type Service = FetchCandlesForward<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FetchCandlesForward {
            svc: Buffer::new(inner.into_service(), self.bound),
            limit: self.limit,
        }
    }
}

/// Fetch candles forward.
pub struct FetchCandlesForward<S>
where
    S: ExchangeService<QueryFirstCandles> + 'static,
{
    svc: Buffer<Exc<S>, QueryFirstCandles>,
    limit: NonZeroUsize,
}

impl<S> Service<QueryCandles> for FetchCandlesForward<S>
where
    S: ExchangeService<QueryFirstCandles> + 'static,
    S::Future: Send,
{
    type Response = CandleStream;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, query: QueryCandles) -> Self::Future {
        let mut query = QueryFirstCandles {
            query,
            first: self.limit.get(),
        };
        let mut svc = self.svc.clone();
        async move {
	    let stream = try_stream!{
		loop {
            tracing::trace!("query candles: {query}");
            if query.query().is_empty() { break; }
		    let mut stream = (&mut svc).oneshot(query.clone()).await.map_err(ExchangeError::Layer)?;
		    let mut next = None;
		    while let Some(c) = stream.next().await {
			let c = c?;
			next = Some(c.ts);
			yield c;
		    }
		    if let Some(next) = next {
			query.query.start = Bound::Excluded(next);
		    } else {
			break;
		    }
		}
	    };
	    Ok(stream.boxed())
	}.boxed()
    }
}
