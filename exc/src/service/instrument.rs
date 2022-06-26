use tower::{util::Oneshot, ServiceExt};

use crate::types::instrument::{FetchInstruments, SubscribeInstruments};

use super::{ExcMut, ExchangeService};

/// Subscribe instruments service.
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

/// Fetch instruments service.
pub trait FetchInstrumentsService: ExchangeService<FetchInstruments> {
    /// Fetch instruments filter by a given tag.
    fn fetch_instruments(&mut self, tag: &str) -> Oneshot<ExcMut<'_, Self>, FetchInstruments>
    where
        Self: Sized,
    {
        ServiceExt::<FetchInstruments>::oneshot(
            self.as_service_mut(),
            FetchInstruments {
                tag: tag.to_string(),
            },
        )
    }
}

impl<S> FetchInstrumentsService for S where S: ExchangeService<FetchInstruments> {}
