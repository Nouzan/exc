use exc_core::{
    types::{SubscribeBidAsk, SubscribeTickers, SubscribeTrades},
    Adaptor, Exc, ExcService, Request,
};
use tower::Layer;

use crate::{TradeBidAsk, TradeBidAskServiceLayer};

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
        Req: Adaptor<SubscribeTrades> + Adaptor<SubscribeBidAsk> + 'static,
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
        Req: Adaptor<SubscribeTrades> + Adaptor<SubscribeBidAsk> + 'static,
    {
        Exc::new(
            TradeBidAskServiceLayer::default()
                .accept_bid_ask_ts()
                .layer(self.into_exc()),
        )
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
