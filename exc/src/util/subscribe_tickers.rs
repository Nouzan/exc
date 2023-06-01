use async_stream::try_stream;
use futures::{future::BoxFuture, FutureExt, StreamExt, TryStreamExt};
use rust_decimal::Decimal;
use std::task::{Context, Poll};
use time::OffsetDateTime;
use tokio_stream::StreamExt as _;
use tower::{Layer, Service, ServiceExt};

use crate::{
    core::types::{
        ticker::{MiniTicker, SubscribeMiniTickers, SubscribeTickers, Ticker, TickerStream},
        BidAsk, MiniTickerStream, SubscribeBidAsk, SubscribeTrades, Trade,
    },
    ExcService, ExchangeError, SubscribeTradesService,
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
        ServiceExt::<SubscribeTickers>::oneshot(self, SubscribeTickers::new(inst)).boxed()
    }
}

/// Subscribe Mini Ticker service.
pub trait SubscribeMiniTickersService {
    /// Subscribe mini tickers.
    fn subscribe_mini_tickers(
        &mut self,
        inst: &str,
    ) -> BoxFuture<'_, crate::Result<MiniTickerStream>>;
}

impl<S> SubscribeMiniTickersService for S
where
    S: ExcService<SubscribeMiniTickers> + Send,
    S::Future: Send,
{
    /// Subscribe mini tickers.
    fn subscribe_mini_tickers(
        &mut self,
        inst: &str,
    ) -> BoxFuture<'_, crate::Result<MiniTickerStream>> {
        ServiceExt::oneshot(self, SubscribeMiniTickers::new(inst)).boxed()
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
    S: ExcService<SubscribeMiniTickers>,
    S: ExcService<SubscribeTrades>,
    S: ExcService<SubscribeBidAsk>,
    <S as Service<SubscribeMiniTickers>>::Future: Send,
    <S as Service<SubscribeTrades>>::Future: Send,
    <S as Service<SubscribeBidAsk>>::Future: Send,
{
    type Response = TickerStream;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<SubscribeTrades>::poll_ready(&mut self.svc, cx)
    }

    fn call(&mut self, req: SubscribeTickers) -> Self::Future {
        enum Response {
            Trade(Trade),
            BidAsk(BidAsk),
            MiniTicker(MiniTicker),
        }

        let mut svc = self.svc.clone();
        let ignore_bid_ask_ts = self.ignore_bid_ask_ts;
        async move {
            let trades = svc
                .subscribe_trades(&req.instrument)
                .await?
                .map_ok(Response::Trade);
            let mini_tickers = svc
                .subscribe_mini_tickers(&req.instrument)
                .await?
                .map_ok(Response::MiniTicker);
            let bid_asks = svc
                .subscribe_bid_ask(&req.instrument)
                .await?
                .map_ok(Response::BidAsk);
            let stream = trades.merge(bid_asks).merge(mini_tickers);
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
                    open_24h:None,
                    high_24h:None,
                    low_24h:None,
                    vol_24h:None,
                };
                let mut trade_init = false;
                for await event in stream {
                    let event = event?;
                    match event {
                        Response::Trade(trade) => {
                            ticker.ts = trade.ts;
                            ticker.last = trade.price;
                            ticker.size = trade.size;
                            ticker.buy = Some(trade.buy);
                            trade_init = true;
                        },
                        Response::MiniTicker(mini_ticker) => {
                            if !ignore_bid_ask_ts {
                                ticker.ts = mini_ticker.ts;
                            }
                            ticker.last = mini_ticker.last;
                            ticker.open_24h = Some(mini_ticker.open_24h);
                            ticker.high_24h = Some(mini_ticker.high_24h);
                            ticker.low_24h = Some(mini_ticker.low_24h);
                            ticker.vol_24h = Some(mini_ticker.vol_24h);
                        },
                        Response::BidAsk(bid_ask) => {
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
