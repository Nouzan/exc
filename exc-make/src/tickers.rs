use exc_service::{ExchangeError, Request, SendExcService};
use exc_types::SubscribeTickers;
use futures::{future::MapErr, TryFutureExt};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower_make::MakeService;

/// How to get the first trade.
#[derive(Debug, Clone, Copy, Default)]
pub enum FirstTrade {
    /// Wait for the first trade.
    #[default]
    Wait,
    /// Use bid for the first trade.
    Bid,
    /// Use ask for the first trade.
    Ask,
    /// Use first bid/ask for the first trade.
    BidAsk,
}

/// Options for making a service to subscribe tickers.
#[derive(Debug, Default)]
pub struct MakeTickersOptions {
    prefer_trade_bid_ask: bool,
    first_trade: FirstTrade,
}

impl MakeTickersOptions {
    /// Set whether to prefer use ticker from trade bid/ask. Default is `false`.
    pub fn prefer_trade_bid_ask(mut self, flag: bool) -> Self {
        self.prefer_trade_bid_ask = flag;
        self
    }

    /// Set how to get the first trade. Default is [`FirstTrade::Wait`].
    /// Only works when [`MakeTickersOptions::prefer_trade_bid_ask`] is `true`.
    pub fn set_first_trade(mut self, mode: FirstTrade) -> Self {
        self.first_trade = mode;
        self
    }

    /// Get whether to prefer use ticker from trade bid/ask.
    pub fn is_prefer_trade_bid_ask(&self) -> bool {
        self.prefer_trade_bid_ask
    }

    /// Get how to get the first trade.
    pub fn first_trade(&self) -> FirstTrade {
        self.first_trade
    }
}

/// Make a service to subscribe tickers.
pub trait MakeTickers {
    /// Service to subscribe tickers.
    type Service: SendExcService<SubscribeTickers>;

    /// The future of the service.
    type Future: Future<Output = Result<Self::Service, ExchangeError>>;

    /// Returns `Ready` when the factory is able to create more service.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>>;

    /// Create a new service to subscribe tickers.
    fn make_tickers(&mut self, options: MakeTickersOptions) -> Self::Future;

    /// Convert to a [`Service`](tower_service::Service).
    fn as_make_tickers_service(&mut self) -> AsService<'_, Self>
    where
        Self: Sized,
    {
        AsService { make: self }
    }
}

impl<M> MakeTickers for M
where
    M: MakeService<
        MakeTickersOptions,
        SubscribeTickers,
        Response = <SubscribeTickers as Request>::Response,
        Error = ExchangeError,
    >,
    M::Service: SendExcService<SubscribeTickers>,
    M::MakeError: Into<ExchangeError>,
{
    type Service = M::Service;

    type Future = MapErr<M::Future, fn(M::MakeError) -> ExchangeError>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), ExchangeError>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn make_tickers(&mut self, options: MakeTickersOptions) -> Self::Future {
        self.make_service(options).map_err(Into::into)
    }
}

crate::create_as_service!(
    MakeTickers,
    MakeTickersOptions,
    make_tickers,
    "Service returns by [`MakeTickers::as_make_tickers_service`]."
);
