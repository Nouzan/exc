use tower::{util::Oneshot, ServiceExt};

use crate::types::subscriptions::SubscribeTickers;

use super::ExchangeService;

/// Subscribe tickers service.
pub trait SubscribeTickersService: ExchangeService<SubscribeTickers> {
    /// Subscribe tickers.
    fn subscribe_tickers(&mut self, inst: &str) -> Oneshot<&mut Self, SubscribeTickers>
    where
        Self: Sized,
    {
        ServiceExt::<SubscribeTickers>::oneshot(self, SubscribeTickers::new(inst))
    }
}

impl<S> SubscribeTickersService for S where S: ExchangeService<SubscribeTickers> {}
