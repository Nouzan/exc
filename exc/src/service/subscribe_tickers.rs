use async_stream::try_stream;
use either::Either;
use futures::{future::BoxFuture, FutureExt, StreamExt, TryStreamExt};
use rust_decimal::Decimal;
use std::task::{Context, Poll};
use time::OffsetDateTime;
use tower::{util::Oneshot, Layer, Service, ServiceExt};

use crate::{
    types::{
        ticker::{SubscribeTickers, Ticker, TickerStream},
        SubscribeBidAsk, SubscribeTrades,
    },
    ExchangeError,
};

use super::{book::SubscribeBidAskService, ExcMut, ExchangeService};

/// Subscribe tickers service.
pub trait SubscribeTickersService: ExchangeService<SubscribeTickers> {
    /// Subscribe tickers.
    fn subscribe_tickers(&mut self, inst: &str) -> Oneshot<ExcMut<'_, Self>, SubscribeTickers>
    where
        Self: Sized,
    {
        ServiceExt::<SubscribeTickers>::oneshot(self.as_service_mut(), SubscribeTickers::new(inst))
    }
}

impl<S> SubscribeTickersService for S where S: ExchangeService<SubscribeTickers> {}

/// Trada-Bid-Ask service.
pub trait TradeBidAskService:
    ExchangeService<SubscribeTrades> + ExchangeService<SubscribeBidAsk> + Clone + Send + 'static
where
    <Self as ExchangeService<SubscribeTrades>>::Future: Send,
    <Self as ExchangeService<SubscribeBidAsk>>::Future: Send,
{
    /// Convert into a [`SubscribeTickersService`].
    fn into_subscribe_tickers(self) -> TradeBidAsk<Self> {
        TradeBidAsk { svc: self }
    }
}

impl<S> TradeBidAskService for S
where
    S: ExchangeService<SubscribeTrades>,
    S: ExchangeService<SubscribeBidAsk>,
    <S as ExchangeService<SubscribeTrades>>::Future: Send,
    <S as ExchangeService<SubscribeBidAsk>>::Future: Send,
    S: Clone + Send + 'static,
{
}

/// Trade-Bid-Ask service layer.
pub struct TradeBidAskServiceLayer;

impl<S> Layer<S> for TradeBidAsk<S> {
    type Service = TradeBidAsk<S>;
    fn layer(&self, inner: S) -> Self::Service {
        TradeBidAsk { svc: inner }
    }
}

/// Trade-Bid-Ask service.
#[derive(Debug, Clone, Copy)]
pub struct TradeBidAsk<S> {
    svc: S,
}

impl<S> Service<SubscribeTickers> for TradeBidAsk<S>
where
    S: Clone + Send + 'static,
    S: ExchangeService<SubscribeTrades>,
    S: ExchangeService<SubscribeBidAsk>,
    <S as ExchangeService<SubscribeTrades>>::Future: Send,
    <S as ExchangeService<SubscribeBidAsk>>::Future: Send,
{
    type Response = TickerStream;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<SubscribeTrades>::poll_ready(
            &mut ExchangeService::<SubscribeTrades>::as_service_mut(&mut self.svc),
            cx,
        )
    }

    fn call(&mut self, req: SubscribeTickers) -> Self::Future {
        let trade = Service::<SubscribeTrades>::call(
            &mut ExchangeService::<SubscribeTrades>::as_service_mut(&mut self.svc),
            SubscribeTrades {
                instrument: req.instrument.clone(),
            },
        );
        let mut svc = self.svc.clone();
        async move {
            let trades = trade.await?.map_ok(Either::Left);
            let bid_asks = svc
                .subscribe_bid_ask(&req.instrument)
                .await?
                .map_ok(Either::Right);
            let stream = tokio_stream::StreamExt::merge(trades, bid_asks);
            let stream = try_stream! {
                let mut ticker = Ticker {
                    ts: OffsetDateTime::now_utc(),
                    last: Decimal::ZERO,
                    size: Decimal::ZERO,
                    buy: None,
                    bid: None,
                    bid_size: None,
                    ask: None,
                    ask_size: None,
                };
                let mut trade_init = false;
                for await event in stream {
                    let event = event?;
                    match event {
                        Either::Left(trade) => {
                            ticker.ts = trade.ts;
                            ticker.last = trade.price;
                            ticker.size = trade.size;
                            ticker.buy = Some(trade.buy);
                            trade_init = true;
                        },
                        Either::Right(bid_ask) => {
                            ticker.ts = bid_ask.ts;
                            ticker.size = Decimal::ZERO;
                            ticker.bid = bid_ask.bid.map(|b| b.0);
                            ticker.ask = bid_ask.ask.map(|a| a.0);
                            ticker.bid_size = bid_ask.bid.map(|b| b.1);
                            ticker.ask_size = bid_ask.ask.map(|a| a.1);
                        }
                    }
                    if trade_init {
                        yield ticker;
                    }
                }
            };
            Ok(stream.boxed())
        }
        .boxed()
    }
}
