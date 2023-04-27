use crate::key::OkxKey as Key;

use super::types::{
    request::HttpRequest,
    response::{FullHttpResponse, HttpResponse},
};
use exc_core::{retry::RetryPolicy, ExchangeError};
use futures::{
    future::{ready, BoxFuture},
    FutureExt, TryFutureExt,
};
use http::{Request, Response};
use hyper::Body;
use tower::{retry::Retry, Layer, Service, ServiceBuilder};

/// Okx HTTP API layer.
pub struct OkxHttpApiLayer<F> {
    testing: bool,
    aws: bool,
    key: Option<Key>,
    retry_policy: RetryPolicy<HttpRequest, HttpResponse, F>,
}

impl<F> OkxHttpApiLayer<F> {
    /// Set key.
    pub fn private(&mut self, key: Key) -> &mut Self {
        self.key = Some(key);
        self
    }

    /// Set whether to use the testing environment.
    pub fn testing(&mut self, enable: bool) -> &mut Self {
        self.testing = enable;
        self
    }

    /// Switch to AWS endpoint.
    pub fn aws(&mut self, enable: bool) -> &mut Self {
        self.aws = enable;
        self
    }

    /// Set retry policy.
    pub fn retry<F2>(
        self,
        policy: RetryPolicy<HttpRequest, HttpResponse, F2>,
    ) -> OkxHttpApiLayer<F2>
    where
        F2: Clone,
    {
        OkxHttpApiLayer {
            aws: self.aws,
            retry_policy: policy,
            key: self.key,
            testing: self.testing,
        }
    }

    /// Get Okx HTTP API Host.
    pub fn host(&self) -> &'static str {
        match (self.testing, self.aws) {
            (true, _) => "https://www.okx.com",
            (false, true) => "https://aws.okx.com",
            (false, false) => "https://www.okx.com",
        }
    }

    /// Retry on `true`.
    pub fn retry_on<F2>(self, f: F2) -> OkxHttpApiLayer<F2>
    where
        F2: Fn(&ExchangeError) -> bool,
        F2: Send + 'static + Clone,
    {
        self.retry(RetryPolicy::default().retry_on(f))
    }

    /// Always retry on errors.
    pub fn retry_on_error(self) -> OkxHttpApiLayer<fn(&ExchangeError) -> bool> {
        self.retry(RetryPolicy::default().retry_on(|_| true))
    }
}

impl Default for OkxHttpApiLayer<fn(&ExchangeError) -> bool> {
    fn default() -> Self {
        Self::new()
    }
}

impl OkxHttpApiLayer<fn(&ExchangeError) -> bool> {
    /// Create a new okx http api layer.
    pub fn new() -> Self {
        Self {
            aws: false,
            retry_policy: RetryPolicy::never(),
            key: None,
            testing: false,
        }
    }
}

impl<S, F> Layer<S> for OkxHttpApiLayer<F>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone,
    S::Future: Send + 'static,
    S::Error: 'static,
    ExchangeError: From<S::Error>,
    F: Fn(&ExchangeError) -> bool,
    F: Send + 'static + Clone,
{
    type Service = Retry<RetryPolicy<HttpRequest, HttpResponse, F>, OkxHttpApi<S>>;

    fn layer(&self, inner: S) -> Self::Service {
        let svc = OkxHttpApi {
            host: self.host().to_string(),
            http: inner,
            key: self.key.clone(),
            testing: self.testing,
        };
        ServiceBuilder::default()
            .retry(self.retry_policy.clone())
            .service(svc)
    }
}

/// Okx HTTP API Service.
#[derive(Clone)]
pub struct OkxHttpApi<S> {
    host: String,
    key: Option<Key>,
    http: S,
    testing: bool,
}

const TESTING_HEADER: &str = "x-simulated-trading";

impl<S> Service<HttpRequest> for OkxHttpApi<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
    S::Future: Send + 'static,
    S::Error: 'static,
    ExchangeError: From<S::Error>,
{
    type Response = HttpResponse;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.http.poll_ready(cx).map_err(ExchangeError::from)
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        let req = match req {
            HttpRequest::Get(get) => serde_qs::to_string(&get)
                .map_err(|err| ExchangeError::Other(err.into()))
                .and_then(|q| {
                    let uri = format!("{}{}?{}", self.host, get.uri(), q);
                    Request::get(uri)
                        .body(Body::empty())
                        .map_err(|err| ExchangeError::Other(err.into()))
                }),
            HttpRequest::PrivateGet(get) => {
                if let Some(key) = self.key.as_ref() {
                    get.to_request(&self.host, key)
                } else {
                    Err(ExchangeError::KeyError(anyhow::anyhow!(
                        "key has not been set"
                    )))
                }
            }
        };
        match req {
            Ok(mut req) => {
                if self.testing {
                    req.headers_mut()
                        .insert(TESTING_HEADER, http::HeaderValue::from_static("1"));
                }
                self.http
                    .call(req)
                    .map_err(ExchangeError::from)
                    .and_then(|resp| {
                        trace!("http response; status: {:?}", resp.status());
                        hyper::body::to_bytes(resp.into_body())
                            .map_err(|err| ExchangeError::Other(err.into()))
                    })
                    .and_then(|bytes| {
                        tracing::trace!(?bytes, "http response;");
                        let resp = serde_json::from_slice::<FullHttpResponse>(&bytes)
                            .map_err(|err| ExchangeError::Other(err.into()));

                        futures::future::ready(resp)
                    })
                    .and_then(|resp| ready(resp.try_into()))
                    .boxed()
            }
            Err(err) => ready(Err(err)).boxed(),
        }
    }
}
