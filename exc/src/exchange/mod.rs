use std::marker::PhantomData;
use tower::{
    buffer::Buffer,
    limit::{rate::Rate, RateLimit},
    util::BoxService,
    Service, ServiceExt,
};

use crate::{types::Request, ExchangeError};

mod subscribe_tickers;

/// Exchange.
#[derive(Debug)]
pub struct Exchange<C, Req> {
    channel: C,
    _req: PhantomData<fn() -> Req>,
}

impl<C: Clone, Req> Clone for Exchange<C, Req> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
            _req: PhantomData,
        }
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

    /// Add rate limit to the given request.
    pub fn rate_limited<R>(
        self,
        rate: Rate,
    ) -> Exchange<RateLimit<BoxService<R, R::Response, ExchangeError>>, R>
    where
        R: Request,
        Self: Service<R, Response = R::Response, Error = ExchangeError>,
        Req: 'static,
        <Self as Service<R>>::Future: Send + 'static,
        C: Send + 'static,
    {
        let svc = self.boxed();
        Exchange::new(RateLimit::new(svc, rate))
    }

    /// Create a buffered exchange.
    pub fn buffered(self, bound: usize) -> Exchange<Buffer<C, Req>, Req>
    where
        C: Service<Req> + Send + 'static,
        <C as Service<Req>>::Future: Send + 'static,
        <C as Service<Req>>::Error: Send + Sync + 'static + std::error::Error,
        Req: Send + 'static,
    {
        Exchange {
            channel: Buffer::new(self.channel, bound),
            _req: PhantomData,
        }
    }
}
