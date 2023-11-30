use crate::Binance;
use exc_core::{
    exchange::{
        MakeCancelOrderOptions, MakeCheckOrderOptions, MakeExchange, MakeFetchCandlesOptions,
        MakeInstrumentsOptions, MakePlaceOrderOptions, MakeSubscribeOrdersOptions,
        MakeTickersOptions,
    },
    service::{BoxCloneExcService, BoxExcService},
    types::{
        CancelOrder, GetOrder, PlaceOrder, QueryCandles, SubscribeInstruments, SubscribeOrders,
        SubscribeTickers,
    },
    util::{
        fetch_candles::FetchCandlesForwardLayer, poll_instruments::PollInstrumentsLayer,
        trade_bid_ask::TradeBidAskLayer,
    },
    ExcServiceExt, ExchangeError, IntoExc,
};
use futures::future::{ready, Ready};
use std::{
    task::{Context, Poll},
    time::Duration,
};
use tower::Service;

/// Binance Exchange.
#[derive(Clone)]
pub struct BinanceExchange {
    name: String,
    inner: Binance,
}

impl BinanceExchange {
    /// Create a new `BinanceExchange`.
    pub fn new(name: &str, inner: Binance) -> Self {
        Self {
            name: name.to_string(),
            inner,
        }
    }
}

impl Service<MakeInstrumentsOptions> for BinanceExchange {
    type Response = BoxCloneExcService<SubscribeInstruments>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeInstrumentsOptions) -> Self::Future {
        let svc = self
            .inner
            .clone()
            .adapt()
            .apply(&PollInstrumentsLayer::new(Duration::from_secs(3600)));
        ready(Ok(svc.boxed_clone()))
    }
}

impl Service<MakeTickersOptions> for BinanceExchange {
    type Response = BoxCloneExcService<SubscribeTickers>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeTickersOptions) -> Self::Future {
        let svc = self.inner.clone().into_exc();
        ready(Ok(ExcServiceExt::<crate::Request>::apply(
            svc,
            &TradeBidAskLayer::default(),
        )
        .boxed_clone()))
    }
}

impl Service<MakePlaceOrderOptions> for BinanceExchange {
    type Response = BoxCloneExcService<PlaceOrder>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakePlaceOrderOptions) -> Self::Future {
        ready(Ok(self.inner.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeCancelOrderOptions> for BinanceExchange {
    type Response = BoxCloneExcService<CancelOrder>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeCancelOrderOptions) -> Self::Future {
        ready(Ok(self.inner.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeCheckOrderOptions> for BinanceExchange {
    type Response = BoxCloneExcService<GetOrder>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeCheckOrderOptions) -> Self::Future {
        ready(Ok(self.inner.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeSubscribeOrdersOptions> for BinanceExchange {
    type Response = BoxCloneExcService<SubscribeOrders>;

    type Error = ExchangeError;

    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: MakeSubscribeOrdersOptions) -> Self::Future {
        ready(Ok(self.inner.clone().adapt().boxed_clone()))
    }
}

impl Service<MakeFetchCandlesOptions> for BinanceExchange {
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
        let (num, per) = rate_limit.unwrap_or((200, Duration::from_secs(60)));
        let limit = batch_limit.unwrap_or(1000);
        ready(Ok(self
            .inner
            .clone()
            .rate_limited(num, per)
            .adapt()
            .apply(&FetchCandlesForwardLayer::with_default_bound(limit))
            .boxed()))
    }
}

impl MakeExchange for BinanceExchange {
    fn name(&self) -> &str {
        &self.name
    }
}
