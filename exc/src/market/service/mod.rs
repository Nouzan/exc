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

use super::MarketOptions;
use super::{
    request::{Kind, Request},
    response::Response,
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

/// Market Service (the inner part).
struct MarketService {
    state: Arc<State>,
    svc_state: ServiceState,
}

impl MarketService {
    fn new(opts: &MarketOptions, inst: SubscribeInstrumentSvc, fetch: FetchInstrumentSvc) -> Self {
        let state = Arc::default();
        Self {
            svc_state: ServiceState::Init(Worker::new(&state, opts, inst, fetch)),
            state,
        }
    }
}

impl Drop for MarketService {
    fn drop(&mut self) {
        if let ServiceState::Running(handle) = std::mem::take(&mut self.svc_state) {
            handle.abort();
        }
    }
}

impl Service<Request> for MarketService {
    type Response = Response;

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
                        let ServiceState::Running(handle) = std::mem::take(&mut self.svc_state) else { unreachable!() };
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

    fn call(&mut self, req: Request) -> Self::Future {
        match req.kind() {
            Kind::GetInstrument(req) => {
                let meta = self.state.clone().get_instrument(req);
                ready(Ok(Response::from(meta))).boxed()
            }
        }
    }
}

/// Market Service.
#[derive(Debug, Clone)]
pub struct Market {
    inner: BoxCloneService<Request, Response, ExchangeError>,
}

impl Service<Request> for Market {
    type Response = Response;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: Request) -> Self::Future {
        self.inner.call(req)
    }
}

/// Market Service Layer.
#[derive(Debug, Clone)]
pub struct MarketLayer<Req, L1 = ExcLayer<Req>, L2 = ExcLayer<Req>> {
    opts: MarketOptions,
    fetch_instruments: L1,
    subscribe_instruments: L2,
    _req: PhantomData<fn() -> Req>,
}

impl<Req> MarketLayer<Req> {
    /// Create a market layer with the given options.
    pub fn with_options(opts: MarketOptions) -> Self {
        Self {
            opts,
            fetch_instruments: ExcLayer::default(),
            subscribe_instruments: ExcLayer::default(),
            _req: PhantomData,
        }
    }
}

impl<Req, L1, L2> MarketLayer<Req, L1, L2> {
    /// Setup fetch instruments layer.
    pub fn fetch_instruments<L>(self, layer: L) -> MarketLayer<Req, L, L2> {
        MarketLayer {
            opts: self.opts,
            fetch_instruments: layer,
            subscribe_instruments: self.subscribe_instruments,
            _req: PhantomData,
        }
    }

    /// Setup subscribe instruments layer.
    pub fn subscribe_instruments<L>(self, layer: L) -> MarketLayer<Req, L1, L> {
        MarketLayer {
            opts: self.opts,
            fetch_instruments: self.fetch_instruments,
            subscribe_instruments: layer,
            _req: PhantomData,
        }
    }
}

impl<Req> Default for MarketLayer<Req> {
    fn default() -> Self {
        Self::with_options(MarketOptions::default())
    }
}

impl<S, Req, L1: Layer<S>, L2: Layer<S>> Layer<S> for MarketLayer<Req, L1, L2>
where
    S: ExcService<Req> + Send + 'static + Clone,
    S::Future: Send + 'static,
    Req: crate::Request + 'static,
    L1::Service: ExcService<FetchInstruments> + Send + 'static,
    <L1::Service as Service<FetchInstruments>>::Future: Send,
    L2::Service: ExcService<SubscribeInstruments> + Send + 'static,
    <L2::Service as Service<SubscribeInstruments>>::Future: Send,
{
    type Service = Market;

    fn layer(&self, svc: S) -> Self::Service {
        let fetch = ServiceBuilder::default()
            .rate_limit(1, Duration::from_secs(1))
            .layer(&self.fetch_instruments)
            .service(svc.clone())
            .boxed();
        let subscribe = ServiceBuilder::default()
            .rate_limit(1, Duration::from_secs(1))
            .layer(&self.subscribe_instruments)
            .service(svc)
            .boxed();
        let svc = MarketService::new(&self.opts, subscribe, fetch);
        let inner = ServiceBuilder::default()
            .buffer(self.opts.buffer_bound)
            .service(svc)
            .map_err(|err| ExchangeError::from(err).flatten())
            .boxed_clone();
        Market { inner }
    }
}
