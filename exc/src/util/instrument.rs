use std::time::Duration;

use async_stream::stream;
use exc_core::types::instrument::InstrumentStream;
use exc_core::{ExchangeError, Str};
use futures::future::{ready, BoxFuture, Ready};
use futures::{FutureExt, StreamExt, TryStreamExt};
use tokio::time::MissedTickBehavior;
use tower::ServiceExt;
use tower::{Layer, Service};

use crate::core::types::instrument::{FetchInstruments, SubscribeInstruments};

use crate::ExcService;

/// Subscribe instruments service.
pub trait SubscribeInstrumentsService {
    /// Subscribe instruments filter by a given tag.
    fn subscribe_instruments(
        &mut self,
        tag: &str,
    ) -> BoxFuture<'_, crate::Result<InstrumentStream>>;
}

impl<S> SubscribeInstrumentsService for S
where
    S: ExcService<SubscribeInstruments> + Send,
    S::Future: Send,
{
    /// Subscribe instruments filter by a given tag.
    fn subscribe_instruments(&mut self, tag: &str) -> BoxFuture<'_, crate::Result<InstrumentStream>>
    where
        Self: Sized,
    {
        ServiceExt::<SubscribeInstruments>::oneshot(
            self,
            SubscribeInstruments { tag: Str::new(tag) },
        )
        .boxed()
    }
}

/// Fetch instruments service.
pub trait FetchInstrumentsService: ExcService<FetchInstruments> {
    /// Fetch instruments filter by a given tag.
    fn fetch_instruments(&mut self, tag: &str) -> BoxFuture<'_, crate::Result<InstrumentStream>>;
}

impl<S> FetchInstrumentsService for S
where
    S: ExcService<FetchInstruments> + Send,
    S::Future: Send,
{
    /// Fetch instruments filter by a given tag.
    fn fetch_instruments(&mut self, tag: &str) -> BoxFuture<'_, crate::Result<InstrumentStream>>
    where
        Self: Sized,
    {
        ServiceExt::<FetchInstruments>::oneshot(self, FetchInstruments { tag: Str::new(tag) })
            .boxed()
    }
}

/// Subscribe instruments by polling.
#[derive(Debug, Clone, Copy)]
pub struct PollInstruments<S> {
    interval: Duration,
    inner: S,
}

impl<S> Service<SubscribeInstruments> for PollInstruments<S>
where
    S: ExcService<FetchInstruments> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = <SubscribeInstruments as crate::Request>::Response;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: SubscribeInstruments) -> Self::Future {
        let mut interval = tokio::time::interval(self.interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let req = stream! {
            loop {
                yield FetchInstruments {
                    tag: req.tag.clone()
                };
                interval.tick().await;
            }
        };
        let stream = self.inner.clone().call_all(req).try_flatten();
        ready(Ok(stream.boxed()))
    }
}

/// Subscribe instruments by polling.
#[derive(Debug, Clone, Copy)]
pub struct PollInstrumentsLayer(Duration);

impl PollInstrumentsLayer {
    /// Create a new poll instruments layer.
    pub fn new(interval: Duration) -> Self {
        Self(interval)
    }
}

impl<S> Layer<S> for PollInstrumentsLayer {
    type Service = PollInstruments<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PollInstruments {
            inner,
            interval: self.0,
        }
    }
}
