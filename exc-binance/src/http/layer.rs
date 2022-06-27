use exc_core::retry::RetryPolicy;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use tower::retry::Retry;

use super::error::RestError;
use super::request::{Payload, Rest, RestEndpoint, RestRequest};
use super::response::{Data, RestResponse};
use std::task::{Context, Poll};
use tower::{Layer, Service, ServiceBuilder};

type Policy = RetryPolicy<RestRequest<Payload>, RestResponse<Data>, fn(&RestError) -> bool>;

/// Binance rest api layer.
#[derive(Clone, Copy)]
pub struct BinanceRestApiLayer {
    retry: Policy,
    endpoint: RestEndpoint,
}

impl BinanceRestApiLayer {
    /// USD-Margin Futures http endpoint.
    pub fn usd_margin_futures() -> Self {
        Self::new(RestEndpoint::UsdMarginFutures)
    }

    /// Create a new binance rest api layer.
    pub fn new(endpoint: RestEndpoint) -> Self {
        Self {
            endpoint,
            retry: RetryPolicy::On {
                f: RestError::is_temporary,
                times: 0,
            },
        }
    }
}

impl<S> Layer<S> for BinanceRestApiLayer {
    type Service = BinanceRestApi<S>;

    fn layer(&self, http: S) -> Self::Service {
        let inner =
            ServiceBuilder::default()
                .retry(self.retry.clone())
                .service(BinanceRestApiInner {
                    http,
                    endpoint: self.endpoint,
                });
        BinanceRestApi { inner }
    }
}

/// Binance rest api service inner part.
#[derive(Clone)]
pub struct BinanceRestApiInner<S> {
    endpoint: RestEndpoint,
    http: S,
}

impl<S, R> Service<RestRequest<R>> for BinanceRestApiInner<S>
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
        match req.to_http(&self.endpoint) {
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

/// Binance rest api service.
#[derive(Clone)]
pub struct BinanceRestApi<S> {
    inner: Retry<Policy, BinanceRestApiInner<S>>,
}

impl<S> Service<RestRequest<Payload>> for BinanceRestApi<S>
where
    S: Clone + Send + 'static,
    S: Service<http::Request<hyper::Body>, Response = http::Response<hyper::Body>>,
    S::Future: Send + 'static,
    S::Error: 'static,
    RestError: From<S::Error>,
{
    type Response = RestResponse<Data>;
    type Error = RestError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: RestRequest<Payload>) -> Self::Future {
        self.inner.call(req).boxed()
    }
}
