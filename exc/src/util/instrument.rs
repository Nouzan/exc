use exc_core::types::instrument::InstrumentStream;
use exc_core::Str;
use futures::future::BoxFuture;
use futures::FutureExt;
use tower::ServiceExt;

use crate::core::types::instrument::{FetchInstruments, SubscribeInstruments};

use crate::ExcService;

/// Subscribe instruments service.
pub trait SubscribeInstrumentsService {
    /// Subscribe instruments filter by a given tag.
    fn subscribe_instruments(
        &mut self,
        tag: &str,
    ) -> BoxFuture<'_, crate::Result<InstrumentStream>>;
}

impl<S> SubscribeInstrumentsService for S
where
    S: ExcService<SubscribeInstruments> + Send,
    S::Future: Send,
{
    /// Subscribe instruments filter by a given tag.
    fn subscribe_instruments(
        &mut self,
        tag: &str,
    ) -> BoxFuture<'_, crate::Result<InstrumentStream>> {
        ServiceExt::<SubscribeInstruments>::oneshot(
            self.as_service(),
            SubscribeInstruments { tag: Str::new(tag) },
        )
        .boxed()
    }
}

/// Fetch instruments service.
pub trait FetchInstrumentsService {
    /// Fetch instruments filter by a given tag.
    fn fetch_instruments(&mut self, tag: &str) -> BoxFuture<'_, crate::Result<InstrumentStream>>;
}

impl<S> FetchInstrumentsService for S
where
    S: ExcService<FetchInstruments> + Send,
    S::Future: Send,
{
    /// Fetch instruments filter by a given tag.
    fn fetch_instruments(&mut self, tag: &str) -> BoxFuture<'_, crate::Result<InstrumentStream>> {
        ServiceExt::<FetchInstruments>::oneshot(
            self.as_service(),
            FetchInstruments { tag: Str::new(tag) },
        )
        .boxed()
    }
}

#[cfg(feature = "poll")]
pub use exc_core::util::poll_instruments::{PollInstruments, PollInstrumentsLayer};
