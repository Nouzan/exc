/// Subscribe tickers.
pub mod subscribe_tickers;

/// Trade.
pub mod trade;

/// Book.
pub mod book;

/// Subscribe instruments.
pub mod instrument;

/// Fetch candles.
pub mod fetch_candles;

/// Trading.
pub mod trading;

/// Reconnect.
pub mod reconnect;

use exc_core::{
    types::{
        QueryCandles, QueryFirstCandles, QueryLastCandles, SubscribeBidAsk, SubscribeStatistics,
        SubscribeTickers, SubscribeTrades,
    },
    Adaptor, Exc, ExcService, Request,
};
use tower::Layer;

use self::{
    fetch_candles::{
        FetchCandlesBackward, FetchCandlesBackwardLayer, FetchCandlesForward,
        FetchCandlesForwardLayer,
    },
    subscribe_tickers::{TradeBidAsk, TradeBidAskServiceLayer},
};

pub use exc_core::util::*;

/// Extension trait of [`Exc`].
pub trait ExcExt<C, Req>: Sized
where
    Req: Request,
    C: ExcService<Req>,
{
    /// Convert into exc.
    fn into_exc(self) -> Exc<C, Req>;

    /// Convert the inner channel to a [`SubscribeTickersService`](crate::SubscribeTickersService)
    fn into_subscribe_tickers(self) -> Exc<TradeBidAsk<Exc<C, Req>>, SubscribeTickers>
    where
        C: Clone + Send + 'static,
        C::Future: Send + 'static,
        Req: Adaptor<SubscribeStatistics>
            + Adaptor<SubscribeTrades>
            + Adaptor<SubscribeBidAsk>
            + 'static,
    {
        Exc::new(TradeBidAskServiceLayer::default().layer(self.into_exc()))
    }

    /// Convert into a [`SubscribeTickersService`](crate::SubscribeTickersService).
    fn into_subscribe_tickers_accpet_bid_ask_ts(
        self,
    ) -> Exc<TradeBidAsk<Exc<C, Req>>, SubscribeTickers>
    where
        C: Clone + Send + 'static,
        C::Future: Send + 'static,
        Req: Adaptor<SubscribeStatistics>
            + Adaptor<SubscribeTrades>
            + Adaptor<SubscribeBidAsk>
            + 'static,
    {
        Exc::new(
            TradeBidAskServiceLayer::default()
                .accept_bid_ask_ts()
                .layer(self.into_exc()),
        )
    }

    /// Convert into a [`FetchCandlesService`](crate::FetchCandlesService)
    /// # Panic
    /// Panic if `limit` is zero.
    fn into_fetch_candles_forward(
        self,
        limit: usize,
    ) -> Exc<FetchCandlesForward<Exc<C, Req>>, QueryCandles>
    where
        Req: Adaptor<QueryFirstCandles>,
        C: Send,
        C::Future: Send,
    {
        Exc::new(FetchCandlesForwardLayer::with_default_bound(limit).layer(self.into_exc()))
    }

    /// Convert into a [`FetchCandlesService`](crate::FetchCandlesService)
    /// # Panic
    /// Panic if `limit` is zero.
    fn into_fetch_candles_forward_with_bound(
        self,
        limit: usize,
        bound: usize,
    ) -> Exc<FetchCandlesForward<Exc<C, Req>>, QueryCandles>
    where
        Req: Adaptor<QueryFirstCandles>,
        C: Send,
        C::Future: Send,
    {
        Exc::new(FetchCandlesForwardLayer::new(limit, bound).layer(self.into_exc()))
    }

    /// Convert into a [`FetchCandlesService`](crate::FetchCandlesService)
    /// # Panic
    /// Panic if `limit` is zero.
    fn into_fetch_candles_backward(
        self,
        limit: usize,
    ) -> Exc<FetchCandlesBackward<Exc<C, Req>>, QueryCandles>
    where
        Req: Adaptor<QueryLastCandles>,
        C: Send,
        C::Future: Send,
    {
        Exc::new(FetchCandlesBackwardLayer::with_default_bound(limit).layer(self.into_exc()))
    }

    /// Convert into a [`FetchCandlesService`](crate::FetchCandlesService)
    /// # Panic
    /// Panic if `limit` is zero.
    fn into_fetch_candles_backward_with_bound(
        self,
        limit: usize,
        bound: usize,
    ) -> Exc<FetchCandlesBackward<Exc<C, Req>>, QueryCandles>
    where
        Req: Adaptor<QueryLastCandles>,
        C: Send,
        C::Future: Send,
    {
        Exc::new(FetchCandlesBackwardLayer::new(limit, bound).layer(self.into_exc()))
    }
}

impl<C, Req> ExcExt<C, Req> for Exc<C, Req>
where
    Req: Request,
    C: ExcService<Req>,
{
    #[inline]
    fn into_exc(self) -> Exc<C, Req> {
        self
    }
}
