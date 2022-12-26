use exc_core::Str;
use tower::{util::Oneshot, ServiceExt};

use crate::types::instrument::{FetchInstruments, SubscribeInstruments};

use crate::ExcService;

/// Subscribe instruments service.
pub trait SubscribeInstrumentsService: ExcService<SubscribeInstruments> {
    /// Subscribe instruments filter by a given tag.
    fn subscribe_instruments(&mut self, tag: &str) -> Oneshot<&mut Self, SubscribeInstruments>
    where
        Self: Sized,
    {
        ServiceExt::<SubscribeInstruments>::oneshot(
            self,
            SubscribeInstruments { tag: Str::new(tag) },
        )
    }
}

impl<S> SubscribeInstrumentsService for S where S: ExcService<SubscribeInstruments> {}

/// Fetch instruments service.
pub trait FetchInstrumentsService: ExcService<FetchInstruments> {
    /// Fetch instruments filter by a given tag.
    fn fetch_instruments(&mut self, tag: &str) -> Oneshot<&mut Self, FetchInstruments>
    where
        Self: Sized,
    {
        ServiceExt::<FetchInstruments>::oneshot(self, FetchInstruments { tag: Str::new(tag) })
    }
}

impl<S> FetchInstrumentsService for S where S: ExcService<FetchInstruments> {}
