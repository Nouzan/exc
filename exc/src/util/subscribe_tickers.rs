use futures::{future::BoxFuture, FutureExt};
use tower::ServiceExt;

use crate::{
    core::types::ticker::{SubscribeTickers, TickerStream},
    ExcService,
};

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
        ServiceExt::<SubscribeTickers>::oneshot(self.as_service(), SubscribeTickers::new(inst))
            .boxed()
    }
}
