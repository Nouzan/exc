use std::marker::PhantomData;

use futures::{future::BoxFuture, FutureExt};
use tower::{Layer, Service};

use crate::{Adaptor, ExcService, ExchangeError, Request};

/// Adapt layer.
#[derive(Debug)]
pub struct AdaptLayer<Req, R>(PhantomData<fn() -> (Req, R)>);

impl<Req, R> Default for AdaptLayer<Req, R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S, Req, R> Layer<S> for AdaptLayer<Req, R> {
    type Service = Adapted<S, Req, R>;

    fn layer(&self, inner: S) -> Self::Service {
        Adapted(inner, PhantomData)
    }
}

/// Adapted channel.
#[derive(Debug)]
pub struct Adapted<S, Req, R>(S, PhantomData<fn() -> (Req, R)>);

impl<S: Clone, Req, R> Clone for Adapted<S, Req, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<C, Req, R> Service<R> for Adapted<C, Req, R>
where
    R: Request,
    R::Response: Send + 'static,
    Req: Adaptor<R>,
    C: ExcService<Req>,
    C::Future: Send + 'static,
{
    type Response = R::Response;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let request = Req::from_request(req);
        match request {
            Ok(req) => {
                let res = self.0.call(req);
                async move {
                    let resp = res.await?;
                    let resp = Req::into_response(resp)?;
                    Ok(resp)
                }
                .left_future()
            }
            Err(err) => futures::future::ready(Err(err)).right_future(),
        }
        .boxed()
    }
}
