use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};

use super::error::RestError;
use super::request::{Rest, RestEndpoint, RestRequest};
use super::response::{Data, RestResponse};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Binance rest api layer.
#[derive(Debug, Clone, Copy)]
pub struct BinanceRestApiLayer {
    endpoint: RestEndpoint,
}

impl BinanceRestApiLayer {
    /// USD-Margin Futures http endpoint.
    pub fn usd_margin_futures() -> Self {
        Self::new(RestEndpoint::UsdMarginFutures)
    }

    /// Create a new binance rest api layer.
    pub fn new(endpoint: RestEndpoint) -> Self {
        Self { endpoint }
    }
}

impl<S> Layer<S> for BinanceRestApiLayer {
    type Service = BinanceRestApi<S>;

    fn layer(&self, http: S) -> Self::Service {
        BinanceRestApi {
            http,
            endpoint: self.endpoint,
        }
    }
}

/// Binance rest api service.
#[derive(Clone)]
pub struct BinanceRestApi<S> {
    endpoint: RestEndpoint,
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
