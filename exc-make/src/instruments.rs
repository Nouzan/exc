use exc_service::{ExcService, ExchangeError, Request};
use exc_types::SubscribeInstruments;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// Options for making a service to subscribe instruments.
#[derive(Debug, Default)]
pub struct MakeInstrumentsOptions {}

/// Make a service to subscribe instruments.
pub trait MakeInstruments {
    /// Service to subscribe instruments.
    type Service: ExcService<SubscribeInstruments>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to subscribe instruments.
    fn make_instruments(&mut self, options: MakeInstrumentsOptions) -> Self::Future;
}

impl<M> MakeInstruments for M
where
    M: MakeService<
        MakeInstrumentsOptions,
        SubscribeInstruments,
        Response = <SubscribeInstruments as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: ExcService<SubscribeInstruments>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_instruments(&mut self, options: MakeInstrumentsOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}
