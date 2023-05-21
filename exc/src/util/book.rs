use exc_core::types::BidAskStream;
use futures::future::BoxFuture;
use futures::FutureExt;
use tower::ServiceExt;

use crate::core::types::SubscribeBidAsk;

use crate::ExcService;

/// Subscribe current best bid and ask service.
pub trait SubscribeBidAskService {
    /// Subscribe current best bid and ask.
    fn subscribe_bid_ask(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<BidAskStream>>;
}

impl<S> SubscribeBidAskService for S
where
    S: ExcService<SubscribeBidAsk> + Send,
    S::Future: Send,
{
    /// Subscribe current best bid and ask.
    fn subscribe_bid_ask(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<BidAskStream>> {
        ServiceExt::oneshot(self, SubscribeBidAsk::new(inst)).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn boxed_bid_ask<'a, S>(svc: S) -> Box<dyn SubscribeBidAskService + 'a>
    where
        S: SubscribeBidAskService + 'a,
    {
        Box::new(svc)
    }

    #[cfg(feature = "binance")]
    #[tokio::test]
    async fn test_box_bid_ask() {
        let binance = crate::Binance::usd_margin_futures().connect_exc();
        std::hint::black_box(boxed_bid_ask(binance));
    }
}
