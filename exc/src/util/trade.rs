use crate::ExcService;
use exc_core::types::{SubscribeTrades, TradeStream};
use futures::future::BoxFuture;
use futures::FutureExt;
use tower::ServiceExt;

/// Subscribe trades service.
pub trait SubscribeTradesService {
    /// Subscribe trades.
    fn subscribe_trades(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<TradeStream>>;
}

impl<S> SubscribeTradesService for S
where
    S: ExcService<SubscribeTrades> + Send,
    S::Future: Send,
{
    fn subscribe_trades(&mut self, inst: &str) -> BoxFuture<'_, crate::Result<TradeStream>>
    where
        Self: Sized,
    {
        ServiceExt::oneshot(
            self,
            SubscribeTrades {
                instrument: inst.to_string(),
            },
        )
        .boxed()
    }
}
