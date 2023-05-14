use async_stream::try_stream;
use either::Either;
use futures::{future::BoxFuture, FutureExt, StreamExt, TryStreamExt};
use rust_decimal::Decimal;
use std::task::{Context, Poll};
use time::OffsetDateTime;
use tower::{Layer, Service, ServiceExt};

use crate::{
    core::types::{
        ticker::{SubscribeTickers, Ticker, TickerStream},
        SubscribeBidAsk, SubscribeTrades,
    },
    ExcService, ExchangeError,
};

use super::book::SubscribeBidAskService;

/// Subscribe tickers service.
pub trait SubscribeTickersService {
    /// Subscribe tickers.
    fn subscribe_tickers(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<TickerStream>>;
}

impl<S> SubscribeTickersService for S
where
    S: ExcService<SubscribeTickers> + Send,
    S::Future: Send,
{
    /// Subscribe tickers.
    fn subscribe_tickers(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<TickerStream>> {
        ServiceExt::<SubscribeTickers>::oneshot(self.as_service_mut(), SubscribeTickers::new(inst))
            .boxed()
    }
}

/// Trade-Bid-Ask service layer.
pub struct TradeBidAskServiceLayer {
    ignore_bid_ask_ts: bool,
}

impl Default for TradeBidAskServiceLayer {
    fn default() -> Self {
        Self {
            ignore_bid_ask_ts: true,
        }
    }
}

impl TradeBidAskServiceLayer {
    /// Accept bid/ask ts.
    pub fn accept_bid_ask_ts(&mut self) -> &mut Self {
        self.ignore_bid_ask_ts = false;
        self
    }
}

impl<S> Layer<S> for TradeBidAskServiceLayer {
    type Service = TradeBidAsk<S>;
    fn layer(&self, inner: S) -> Self::Service {
        TradeBidAsk {
            svc: inner,
            ignore_bid_ask_ts: self.ignore_bid_ask_ts,
        }
    }
}

/// Trade-Bid-Ask service.
#[derive(Debug, Clone, Copy)]
pub struct TradeBidAsk<S> {
    ignore_bid_ask_ts: bool,
    svc: S,
}

impl<S> Service<SubscribeTickers> for TradeBidAsk<S>
where
    S: Clone + Send + 'static,
    S: ExcService<SubscribeTrades>,
    S: ExcService<SubscribeBidAsk>,
    <S as Service<SubscribeTrades>>::Future: Send,
    <S as Service<SubscribeBidAsk>>::Future: Send,
{
    type Response = TickerStream;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<SubscribeTrades>::poll_ready(
            &mut ExcService::<SubscribeTrades>::as_service_mut(&mut self.svc),
            cx,
        )
    }

    fn call(&mut self, req: SubscribeTickers) -> Self::Future {
        let trade = Service::<SubscribeTrades>::call(
            &mut ExcService::<SubscribeTrades>::as_service_mut(&mut self.svc),
            SubscribeTrades {
                instrument: req.instrument.clone(),
            },
        );
        let mut svc = self.svc.clone();
        let ignore_bid_ask_ts = self.ignore_bid_ask_ts;
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
                            if !ignore_bid_ask_ts {
                                ticker.ts = bid_ask.ts;
                            }
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
