use std::task::{Context, Poll};

use exc_core::exchange::{
    MakeCancelOrderOptions, MakeCheckOrderOptions, MakeExchange, MakeInstrumentsOptions,
    MakePlaceOrderOptions, MakeSubscribeOrdersOptions, MakeTickersOptions,
};
use exc_core::service::BoxCloneExcService;
use exc_core::types::{
    CancelOrder, GetOrder, PlaceOrder, SubscribeInstruments, SubscribeOrders, SubscribeTickers,
};
use exc_core::{ExcServiceExt, ExchangeError};
use futures::future::{ready, Ready};
use tower::Service;

use crate::service::Okx;

/// Okx Exchange.
#[derive(Clone)]
pub struct OkxExchange {
    public: Okx,
    private: Okx,
}

impl OkxExchange {
    /// Create a new `OkxExchange`.
    pub fn new(public: Okx, private: Okx) -> Self {
        Self { public, private }
    }
}

impl Service<MakeInstrumentsOptions> for OkxExchange {
    type Response = BoxCloneExcService<SubscribeInstruments>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeInstrumentsOptions) -> Self::Future {
        ready(Ok(self.public.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeTickersOptions> for OkxExchange {
    type Response = BoxCloneExcService<SubscribeTickers>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeTickersOptions) -> Self::Future {
        ready(Ok(self.public.clone().adapt().boxed_clone()))
    }
}

impl Service<MakePlaceOrderOptions> for OkxExchange {
    type Response = BoxCloneExcService<PlaceOrder>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakePlaceOrderOptions) -> Self::Future {
        ready(Ok(self.private.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeCancelOrderOptions> for OkxExchange {
    type Response = BoxCloneExcService<CancelOrder>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeCancelOrderOptions) -> Self::Future {
        ready(Ok(self.private.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeCheckOrderOptions> for OkxExchange {
    type Response = BoxCloneExcService<GetOrder>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeCheckOrderOptions) -> Self::Future {
        ready(Ok(self.private.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeSubscribeOrdersOptions> for OkxExchange {
    type Response = BoxCloneExcService<SubscribeOrders>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeSubscribeOrdersOptions) -> Self::Future {
        ready(Ok(self.private.clone().adapt().boxed_clone()))
    }
}

impl MakeExchange for OkxExchange {
    fn name(&self) -> &str {
        "okx"
    }
}
