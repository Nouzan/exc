use std::marker::PhantomData;
use tower::{Service, ServiceExt};

mod subscribe_tickers;

/// Exchange.
#[derive(Debug)]
pub struct Exchange<C, Req> {
    channel: C,
    _req: PhantomData<fn() -> Req>,
}

impl<C, Req> Clone for Exchange<C, Req>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.channel.clone())
    }
}

impl<C, Req> Exchange<C, Req> {
    /// Create a new exchange client from the given channel.
    pub fn new(channel: C) -> Self {
        Self {
            channel,
            _req: PhantomData,
        }
    }

    /// Make a request using the underlying channel directly.
    pub async fn request(&mut self, request: Req) -> Result<C::Response, C::Error>
    where
        C: Service<Req>,
    {
        ServiceExt::<Req>::oneshot(&mut self.channel, request).await
    }
}
