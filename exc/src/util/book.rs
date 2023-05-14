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
        ServiceExt::oneshot(
            self,
            SubscribeBidAsk {
                instrument: inst.to_string(),
            },
        )
        .boxed()
    }
}
