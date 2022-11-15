use exc_core::retry::RetryPolicy;
use exc_core::transport::http::channel::HttpsChannel;
use exc_core::{ExchangeError, Request};
use futures::future::{ready, BoxFuture};
use futures::{FutureExt, TryFutureExt};
use tower::buffer::Buffer;
use tower::ready_cache::ReadyCache;
use tower::retry::Retry;
use tower::util::Either;
use tower::Service;

use crate::http::layer::OkxHttpApi;
use crate::http::types::{request::HttpRequest, response::HttpResponse};
use crate::websocket::transport::channel::Channel as WsChannel;
use crate::websocket::{Request as WsRequest, Response as WsResponse};

use self::endpoint::Endpoint;

/// Endpoint.
pub mod endpoint;

mod adaptation;

/// Okx request.
pub enum OkxRequest {
    /// Request of HTTP API.
    Http(HttpRequest),
    /// Request of WS API.
    Ws(WsRequest),
}

impl OkxRequest {
    /// Subscribe to orders channel.
    pub fn subscribe_orders(inst: &str) -> Self {
        Self::Ws(WsRequest::subscribe_orders(inst))
    }
}

/// Okx response.
pub enum OkxResponse {
    /// Response from HTTP API.
    Http(HttpResponse),
    /// Response from WS API.
    Ws(WsResponse),
}

impl OkxResponse {
    /// Convert into http response.
    pub fn http(self) -> Result<HttpResponse, ExchangeError> {
        if let Self::Http(res) = self {
            Ok(res)
        } else {
            Err(ExchangeError::Other(anyhow::anyhow!(
                "unexpected response type `ws`"
            )))
        }
    }

    /// Convert into websocket response.
    pub fn ws(self) -> Result<WsResponse, ExchangeError> {
        if let Self::Ws(res) = self {
            Ok(res)
        } else {
            Err(ExchangeError::Other(anyhow::anyhow!(
                "unexpected response type `http`"
            )))
        }
    }
}

impl Request for OkxRequest {
    type Response = OkxResponse;
}

type HttpInner = OkxHttpApi<HttpsChannel>;
type Http = Retry<RetryPolicy<HttpRequest, HttpResponse, fn(&ExchangeError) -> bool>, HttpInner>;
type Ws = WsChannel;

impl Service<OkxRequest> for Http {
    type Response = OkxResponse;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::<HttpRequest>::poll_ready(self, cx)
    }

    fn call(&mut self, req: OkxRequest) -> Self::Future {
        if let OkxRequest::Http(req) = req {
            Service::call(self, req).map_ok(OkxResponse::Http).boxed()
        } else {
            ready(Err(ExchangeError::Other(anyhow::anyhow!(
                "Invalid request type"
            ))))
            .boxed()
        }
    }
}

impl Service<OkxRequest> for Ws {
    type Response = OkxResponse;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::<WsRequest>::poll_ready(self, cx)
    }

    fn call(&mut self, req: OkxRequest) -> Self::Future {
        if let OkxRequest::Ws(req) = req {
            Service::call(self, req).map_ok(OkxResponse::Ws).boxed()
        } else {
            ready(Err(ExchangeError::Other(anyhow::anyhow!(
                "Invalid request type"
            ))))
            .boxed()
        }
    }
}

struct Inner {
    svcs: ReadyCache<&'static str, Either<Http, Ws>, OkxRequest>,
}

const HTTP_KEY: &str = "http";
const WS_KEY: &str = "ws";

impl Inner {
    fn new(ws: Ws, http: Http) -> Self {
        let mut svcs = ReadyCache::default();
        svcs.push(WS_KEY, Either::B(ws));
        svcs.push(HTTP_KEY, Either::A(http));
        Inner { svcs }
    }
}

impl Service<OkxRequest> for Inner {
    type Response = OkxResponse;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svcs
            .poll_pending(cx)
            .map_err(|err| ExchangeError::Unavailable(err.into()))
    }

    fn call(&mut self, req: OkxRequest) -> Self::Future {
        let key = match &req {
            OkxRequest::Http(_) => HTTP_KEY,
            OkxRequest::Ws(_) => WS_KEY,
        };
        self.svcs
            .call_ready(&key, req)
            .map_err(ExchangeError::layer)
            .boxed()
    }
}

/// Okx API.
#[derive(Clone)]
pub struct Okx {
    inner: Buffer<Inner, OkxRequest>,
}

impl Okx {
    fn new(ws: Ws, http: Http, cap: usize) -> Self {
        Self {
            inner: Buffer::new(Inner::new(ws, http), cap),
        }
    }

    /// Create a default endpoint (the [`Okx`] builder).
    pub fn endpoint() -> Endpoint {
        Endpoint::default()
    }
}

impl Service<OkxRequest> for Okx {
    type Response = OkxResponse;

    type Error = ExchangeError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(ExchangeError::layer)
    }

    fn call(&mut self, req: OkxRequest) -> Self::Future {
        self.inner.call(req).map_err(ExchangeError::layer).boxed()
    }
}
