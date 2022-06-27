use tower::{util::Oneshot, ServiceExt};

use crate::types::trade::SubscribeTrades;

use crate::{ExcMut, ExchangeService};

/// Subscribe trades service.
pub trait SubscribeTradesService: ExchangeService<SubscribeTrades> {
    /// Subscribe trades.
    fn subscribe_trades(&mut self, inst: &str) -> Oneshot<ExcMut<'_, Self>, SubscribeTrades>
    where
        Self: Sized,
    {
        ServiceExt::oneshot(
            self.as_service_mut(),
            SubscribeTrades {
                instrument: inst.to_string(),
            },
        )
    }
}

impl<S> SubscribeTradesService for S where S: ExchangeService<SubscribeTrades> {}
