use tower::{util::Oneshot, ServiceExt};

use crate::types::trade::SubscribeTrades;

use crate::ExcService;

/// Subscribe trades service.
pub trait SubscribeTradesService: ExcService<SubscribeTrades> {
    /// Subscribe trades.
    fn subscribe_trades(&mut self, inst: &str) -> Oneshot<&mut Self, SubscribeTrades>
    where
        Self: Sized,
    {
        ServiceExt::oneshot(
            self,
            SubscribeTrades {
                instrument: inst.to_string(),
            },
        )
    }
}

impl<S> SubscribeTradesService for S where S: ExcService<SubscribeTrades> {}
