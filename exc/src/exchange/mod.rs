use futures::{future::BoxFuture, FutureExt};
use std::marker::PhantomData;
use tower::{Service, ServiceExt};

use crate::{types::Request, ExchangeError};

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

impl<C, Req, R> Service<R> for Exchange<C, Req>
where
    C: Service<Req, Error = ExchangeError>,
    Req: TryFrom<R, Error = ExchangeError>,
    R: Request,
    R::Response: TryFrom<C::Response, Error = ExchangeError> + Send + 'static,
    C::Future: Send + 'static,
{
    type Response = R::Response;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.channel.poll_ready(cx).map_err(ExchangeError::from)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let request = Req::try_from(req);
        match request {
            Ok(req) => {
                let res = self.channel.call(req);
                async move {
                    let resp = res.await?;
                    let resp = R::Response::try_from(resp)?;
                    Ok(resp)
                }
                .left_future()
            }
            Err(err) => futures::future::ready(Err(err)).right_future(),
        }
        .boxed()
    }
}
