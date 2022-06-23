use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};

use super::error::RestError;
use super::request::{Rest, RestRequest};
use super::response::{Data, RestResponse};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Binance rest api layer.
pub struct BinanceRestApiLayer;

impl<S> Layer<S> for BinanceRestApiLayer {
    type Service = BinanceRestApi<S>;

    fn layer(&self, http: S) -> Self::Service {
        BinanceRestApi { http }
    }
}

/// Binance rest api service.
#[derive(Clone)]
pub struct BinanceRestApi<S> {
    http: S,
}

impl<S, R> Service<RestRequest<R>> for BinanceRestApi<S>
where
    R: Rest,
    S: Service<http::Request<hyper::Body>, Response = http::Response<hyper::Body>>,
    S::Future: Send + 'static,
    S::Error: 'static,
    RestError: From<S::Error>,
{
    type Response = RestResponse<Data>;
    type Error = RestError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RestRequest<R>) -> Self::Future {
        match req.to_http() {
            Ok(req) => self
                .http
                .call(req)
                .map_err(RestError::from)
                .and_then(|resp| RestResponse::from_http(resp))
                .boxed(),
            Err(err) => futures::future::ready(Err(err)).boxed(),
        }
    }
}
