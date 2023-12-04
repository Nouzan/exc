use std::time::Duration;

use async_stream::stream;
use exc_service::{ExcService, ExcServiceExt, ExchangeError};
use exc_types::{FetchInstruments, SubscribeInstruments};
use futures::{
    future::{ready, Ready},
    StreamExt, TryStreamExt,
};
use tokio::time::MissedTickBehavior;
use tower::{Layer, Service, ServiceExt};

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
        let stream = self
            .inner
            .clone()
            .into_service()
            .call_all(req)
            .try_flatten();
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
