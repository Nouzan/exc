use std::task::{Context, Poll};

use async_stream::try_stream;
use either::Either;
use exc_make::tickers::FirstTrade;
use exc_service::{ExcService, ExchangeError};
use exc_types::{SubscribeBidAsk, SubscribeTickers, SubscribeTrades, Ticker, TickerStream};
use futures::{future::BoxFuture, FutureExt, StreamExt, TryStreamExt};
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tower::{Layer, Service, ServiceExt};

/// Trade-Bid-Ask layer.
pub struct TradeBidAskLayer {
    ignore_bid_ask_ts: bool,
    first_trade: FirstTrade,
}

impl Default for TradeBidAskLayer {
    fn default() -> Self {
        Self {
            ignore_bid_ask_ts: true,
            first_trade: FirstTrade::default(),
        }
    }
}

impl TradeBidAskLayer {
    /// Accept bid/ask ts.
    pub fn accept_bid_ask_ts(&mut self) -> &mut Self {
        self.ignore_bid_ask_ts = false;
        self
    }

    /// Set first trade mode.
    pub fn first_trade(&mut self, mode: FirstTrade) -> &mut Self {
        self.first_trade = mode;
        self
    }
}

impl<S> Layer<S> for TradeBidAskLayer {
    type Service = TradeBidAsk<S>;
    fn layer(&self, inner: S) -> Self::Service {
        TradeBidAsk {
            svc: inner,
            ignore_bid_ask_ts: self.ignore_bid_ask_ts,
            first_trade: self.first_trade,
        }
    }
}

/// Trade-Bid-Ask service.
#[derive(Debug, Clone, Copy)]
pub struct TradeBidAsk<S> {
    ignore_bid_ask_ts: bool,
    first_trade: FirstTrade,
    svc: S,
}

impl<S> Service<SubscribeTickers> for TradeBidAsk<S>
where
    S: Clone + Send + 'static,
    S: ExcService<SubscribeTrades>,
    S: ExcService<SubscribeBidAsk>,
    <S as ExcService<SubscribeTrades>>::Future: Send,
    <S as ExcService<SubscribeBidAsk>>::Future: Send,
{
    type Response = TickerStream;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<SubscribeTrades>::poll_ready(&mut self.svc.as_service(), cx)
    }

    fn call(&mut self, req: SubscribeTickers) -> Self::Future {
        let trade = Service::<SubscribeTrades>::call(
            &mut self.svc.as_service(),
            SubscribeTrades {
                instrument: req.instrument.clone(),
            },
        );
        let mut svc = self.svc.clone();
        let ignore_bid_ask_ts = self.ignore_bid_ask_ts;
        let mode = self.first_trade;
        async move {
            let trades = trade.await?.map_ok(Either::Left);
            let mut svc = svc.as_service();
            let svc = svc.ready().await?;
            let bid_asks = Service::call(
                svc,
                SubscribeBidAsk {
                    instrument: req.instrument,
                },
            )
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
                            if !trade_init {
                                match mode {
                                    FirstTrade::Wait => {},
                                    FirstTrade::Bid => {
                                        if let Some(bid) = bid_ask.bid {
                                            ticker.last = bid.0;
                                            ticker.buy = None;
                                            trade_init = true;
                                        }
                                    },
                                    FirstTrade::Ask => {
                                        if let Some(ask) = bid_ask.ask {
                                            ticker.last = ask.0;
                                            ticker.buy = None;
                                            trade_init = true;
                                        }
                                    },
                                    FirstTrade::BidAsk => {
                                        if let Some(bid) = ticker.bid {
                                            ticker.last = bid;
                                            ticker.buy = None;
                                            trade_init = true;
                                        } else if let Some(ask) = ticker.ask {
                                            ticker.last = ask;
                                            ticker.buy = None;
                                            trade_init = true;
                                        }
                                    },
                                }
                            }
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
