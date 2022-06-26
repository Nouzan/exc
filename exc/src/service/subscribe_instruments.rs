use tower::{util::Oneshot, ServiceExt};

use crate::types::instrument::SubscribeInstruments;

use super::{ExcMut, ExchangeService};

/// Subscribe tickers service.
pub trait SubscribeInstrumentsService: ExchangeService<SubscribeInstruments> {
    /// Subscribe instruments filter by a given tag.
    fn subscribe_instruments(
        &mut self,
        tag: &str,
    ) -> Oneshot<ExcMut<'_, Self>, SubscribeInstruments>
    where
        Self: Sized,
    {
        ServiceExt::<SubscribeInstruments>::oneshot(
            self.as_service_mut(),
            SubscribeInstruments {
                tag: tag.to_string(),
            },
        )
    }
}

impl<S> SubscribeInstrumentsService for S where S: ExchangeService<SubscribeInstruments> {}
