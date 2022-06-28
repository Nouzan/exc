use tower::{util::Oneshot, ServiceExt};

use crate::types::SubscribeBidAsk;

use crate::{ExcMut, ExcService};

/// Subscribe current best bid and ask service.
pub trait SubscribeBidAskService: ExcService<SubscribeBidAsk> {
    /// Subscribe current best bid and ask.
    fn subscribe_bid_ask(&mut self, inst: &str) -> Oneshot<ExcMut<'_, Self>, SubscribeBidAsk>
    where
        Self: Sized,
    {
        ServiceExt::oneshot(
            self.as_service_mut(),
            SubscribeBidAsk {
                instrument: inst.to_string(),
            },
        )
    }
}

impl<S> SubscribeBidAskService for S where S: ExcService<SubscribeBidAsk> {}
