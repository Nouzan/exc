use crate::service::ExchangeService;
use crate::types::BoxData;
use futures::future;

/// Exchange.
#[derive(Debug, Clone, Copy)]
pub struct Exchange<T> {
    inner: T,
}

impl<T> Exchange<T> {
    /// Create a new exchange client with the provied [`ExchangeService`].
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Check if the inner service is ready.
    pub async fn ready(&mut self) -> Result<(), T::Error>
    where
        T: ExchangeService<BoxData>,
    {
        trace!("exchange poll ready");
        future::poll_fn(|cx| self.inner.poll_ready(cx)).await
    }
}
