use std::task::{Context, Poll};
use std::time::Duration;

use exc_core::exchange::{
    MakeCancelOrderOptions, MakeCheckOrderOptions, MakeExchange, MakeFetchCandlesOptions,
    MakeInstrumentsOptions, MakePlaceOrderOptions, MakeSubscribeOrdersOptions, MakeTickersOptions,
};
use exc_core::service::{BoxCloneExcService, BoxExcService};
use exc_core::types::{
    CancelOrder, GetOrder, PlaceOrder, QueryCandles, SubscribeInstruments, SubscribeOrders,
    SubscribeTickers,
};
use exc_core::util::fetch_candles::FetchCandlesBackwardLayer;
use exc_core::util::fetch_instruments_first::FetchThenSubscribeInstrumentsLayer;
use exc_core::util::trade_bid_ask::TradeBidAskLayer;
use exc_core::{ExcServiceExt, ExchangeError, IntoExc};
use futures::future::{ready, Ready};
use tower::{Layer, Service};

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
        ready(Ok(FetchThenSubscribeInstrumentsLayer
            .layer(self.public.clone().into_exc())
            .boxed_clone()))
    }
}

impl Service<MakeTickersOptions> for OkxExchange {
    type Response = BoxCloneExcService<SubscribeTickers>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: MakeTickersOptions) -> Self::Future {
        if req.is_prefer_trade_bid_ask() {
            let svc = self.public.clone().into_exc();
            ready(Ok(ExcServiceExt::<crate::OkxRequest>::apply(
                svc,
                &TradeBidAskLayer::default(),
            )
            .boxed_clone()))
        } else {
            ready(Ok(self.public.clone().adapt().boxed_clone()))
        }
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

impl Service<MakeFetchCandlesOptions> for OkxExchange {
    type Response = BoxExcService<QueryCandles>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: MakeFetchCandlesOptions) -> Self::Future {
        let MakeFetchCandlesOptions {
            rate_limit,
            batch_limit,
        } = req;
        let (num, per) = rate_limit.unwrap_or((19, Duration::from_secs(1)));
        let limit = batch_limit.unwrap_or(100);
        ready(Ok(self
            .public
            .clone()
            .rate_limited(num, per)
            .adapt()
            .apply(&FetchCandlesBackwardLayer::with_default_bound(limit))
            .boxed()))
    }
}

impl MakeExchange for OkxExchange {
    fn name(&self) -> &str {
        "okx"
    }
}
