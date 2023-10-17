use std::time::Duration;
use std::{marker::PhantomData, sync::Arc, task::Poll};

use crate::core::ExcService;
use crate::{core::types::instrument::SubscribeInstruments, ExchangeError};
use exc_core::types::instrument::FetchInstruments;
use exc_core::ExcLayer;
use futures::{
    future::{ready, BoxFuture},
    FutureExt, TryFutureExt,
};
use tokio::task::JoinHandle;
use tower::util::BoxCloneService;
use tower::{util::BoxService, Layer, Service, ServiceBuilder, ServiceExt};

use self::{state::State, worker::Worker};

use self::options::InstrumentsOptions;
use super::{
    request::{InstrumentsRequest, Kind},
    response::InstrumentsResponse,
};

type SubscribeInstrumentSvc = BoxService<
    SubscribeInstruments,
    <SubscribeInstruments as crate::Request>::Response,
    ExchangeError,
>;

type FetchInstrumentSvc =
    BoxService<FetchInstruments, <FetchInstruments as crate::Request>::Response, ExchangeError>;

mod state;
mod worker;

/// Options.
pub mod options;

#[derive(Default)]
enum ServiceState {
    Init(Worker),
    Running(JoinHandle<Result<(), ExchangeError>>),
    Closing(JoinHandle<Result<(), ExchangeError>>),
    #[default]
    Failed,
}

/// Instrument Service (the inner part).
struct Inner {
    state: Arc<State>,
    svc_state: ServiceState,
}

impl Inner {
    fn new(
        opts: &InstrumentsOptions,
        inst: SubscribeInstrumentSvc,
        fetch: FetchInstrumentSvc,
    ) -> Self {
        let state = Arc::default();
        Self {
            svc_state: ServiceState::Init(Worker::new(&state, opts, inst, fetch)),
            state,
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let ServiceState::Running(handle) = std::mem::take(&mut self.svc_state) {
            handle.abort();
        }
    }
}

impl Service<InstrumentsRequest> for Inner {
    type Response = InstrumentsResponse;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            match &mut self.svc_state {
                ServiceState::Init(worker) => {
                    tracing::trace!("init; wait init");
                    futures::ready!(worker.poll_init(cx))?;
                    tracing::trace!("init; spawn worker task");
                    let ServiceState::Init(worker) = std::mem::take(&mut self.svc_state) else {
                        unreachable!();
                    };
                    let handle = tokio::spawn(
                        worker
                            .start()
                            .inspect_err(|err| tracing::error!(%err, "market worker error")),
                    );
                    self.svc_state = ServiceState::Running(handle);
                    break;
                }
                ServiceState::Running(handle) => {
                    if handle.is_finished() {
                        tracing::trace!("running; found finished");
                        let ServiceState::Running(handle) = std::mem::take(&mut self.svc_state)
                        else {
                            unreachable!()
                        };
                        self.svc_state = ServiceState::Closing(handle);
                    } else {
                        tracing::trace!("running; ready");
                        break;
                    }
                }
                ServiceState::Closing(handle) => {
                    tracing::trace!("closing; closing");
                    match handle.try_poll_unpin(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(res) => {
                            self.svc_state = ServiceState::Failed;
                            res.map_err(|err| ExchangeError::Other(err.into()))
                                .and_then(|res| res)?;
                        }
                    }
                }
                ServiceState::Failed => {
                    tracing::trace!("failed; failed");
                    return Poll::Ready(Err(ExchangeError::Other(anyhow::anyhow!(
                        "market worker dead"
                    ))));
                }
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: InstrumentsRequest) -> Self::Future {
        match req.kind() {
            Kind::GetInstrument(req) => {
                let meta = self.state.clone().get_instrument(req);
                ready(Ok(InstrumentsResponse::from(meta))).boxed()
            }
        }
    }
}

/// Instruments Service.
#[derive(Debug, Clone)]
pub struct Instruments {
    inner: BoxCloneService<InstrumentsRequest, InstrumentsResponse, ExchangeError>,
}

impl Service<InstrumentsRequest> for Instruments {
    type Response = InstrumentsResponse;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: InstrumentsRequest) -> Self::Future {
        self.inner.call(req)
    }
}

/// Instruments Service Layer.
#[derive(Debug, Clone)]
pub struct InstrumentsLayer<Req, L1 = ExcLayer<Req>, L2 = ExcLayer<Req>> {
    opts: InstrumentsOptions,
    fetch_instruments: L1,
    subscribe_instruments: L2,
    _req: PhantomData<fn() -> Req>,
}

impl<Req> InstrumentsLayer<Req> {
    /// Create a instruments layer with the default buffer bound.
    pub fn new(inst_tags: &[&str]) -> Self {
        let opts = InstrumentsOptions::default().tags(inst_tags);
        Self {
            opts,
            fetch_instruments: ExcLayer::default(),
            subscribe_instruments: ExcLayer::default(),
            _req: PhantomData,
        }
    }

    /// Create a instruments layer with the given buffer bound.
    pub fn with_buffer_bound(inst_tags: &[&str], bound: usize) -> Self {
        let opts = InstrumentsOptions::default()
            .tags(inst_tags)
            .buffer_bound(bound);
        Self {
            opts,
            fetch_instruments: ExcLayer::default(),
            subscribe_instruments: ExcLayer::default(),
            _req: PhantomData,
        }
    }
}

impl<Req, L1, L2> InstrumentsLayer<Req, L1, L2> {
    /// Setup fetch instruments layer.
    pub fn fetch_instruments<L>(self, layer: L) -> InstrumentsLayer<Req, L, L2> {
        InstrumentsLayer {
            opts: self.opts,
            fetch_instruments: layer,
            subscribe_instruments: self.subscribe_instruments,
            _req: PhantomData,
        }
    }

    /// Setup subscribe instruments layer.
    pub fn subscribe_instruments<L>(self, layer: L) -> InstrumentsLayer<Req, L1, L> {
        InstrumentsLayer {
            opts: self.opts,
            fetch_instruments: self.fetch_instruments,
            subscribe_instruments: layer,
            _req: PhantomData,
        }
    }

    /// Set fetch rate-limit.
    /// Default to 1 per secs.
    pub fn set_fetch_rate_limit(&mut self, num: u64, dur: Duration) -> &mut Self {
        self.opts.fetch_rate_limit = (num, dur);
        self
    }

    /// Set subscribe rate-limit.
    /// Default to 1 per secs.
    pub fn set_subscribe_rate_limit(&mut self, num: u64, dur: Duration) -> &mut Self {
        self.opts.subscribe_rate_limit = (num, dur);
        self
    }
}

impl<S, Req, L1: Layer<S>, L2: Layer<S>> Layer<S> for InstrumentsLayer<Req, L1, L2>
where
    S: ExcService<Req> + Send + 'static + Clone,
    S::Future: Send + 'static,
    Req: crate::Request + 'static,
    L1::Service: ExcService<FetchInstruments> + Send + 'static,
    <L1::Service as Service<FetchInstruments>>::Future: Send,
    L2::Service: ExcService<SubscribeInstruments> + Send + 'static,
    <L2::Service as Service<SubscribeInstruments>>::Future: Send,
{
    type Service = Instruments;

    fn layer(&self, svc: S) -> Self::Service {
        let fetch = ServiceBuilder::default()
            .rate_limit(self.opts.fetch_rate_limit.0, self.opts.fetch_rate_limit.1)
            .layer(&self.fetch_instruments)
            .service(svc.clone())
            .boxed();
        let subscribe = ServiceBuilder::default()
            .rate_limit(
                self.opts.subscribe_rate_limit.0,
                self.opts.subscribe_rate_limit.1,
            )
            .layer(&self.subscribe_instruments)
            .service(svc)
            .boxed();
        let svc = Inner::new(&self.opts, subscribe, fetch);
        let inner = ServiceBuilder::default()
            .buffer(self.opts.buffer_bound)
            .service(svc)
            .map_err(|err| ExchangeError::from(err).flatten())
            .boxed_clone();
        Instruments { inner }
    }
}
