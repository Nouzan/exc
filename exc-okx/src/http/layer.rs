use crate::key::Key;

use super::types::{
    request::HttpRequest,
    response::{FullHttpResponse, HttpResponse},
};
use exc::ExchangeError;
use futures::{
    future::{ready, BoxFuture},
    FutureExt, TryFutureExt,
};
use http::{Request, Response};
use hyper::Body;
use tower::{
    retry::{Policy, Retry},
    Layer, Service, ServiceBuilder,
};

/// Retry Policy.
#[derive(Debug, Clone, Copy)]
pub enum RetryPolicy<F> {
    /// On.
    On {
        /// Error filter.
        f: F,
        /// Rety times.
        times: usize,
    },
    /// Never.
    Never,
}

impl<F> Policy<HttpRequest, HttpResponse, ExchangeError> for RetryPolicy<F>
where
    F: Fn(&ExchangeError) -> bool,
    F: Send + 'static + Clone,
{
    type Future = BoxFuture<'static, Self>;

    fn retry(
        &self,
        _req: &HttpRequest,
        result: Result<&HttpResponse, &ExchangeError>,
    ) -> Option<Self::Future> {
        match self {
            Self::On { f, times } => match result {
                Ok(_) => None,
                Err(err) => {
                    if f(err) {
                        let times = *times;
                        let secs = (1 << times).min(128);
                        trace!("retry in {secs}s; err={err}");
                        let retry = Self::On {
                            f: f.clone(),
                            times: times + 1,
                        };
                        let fut = async move {
                            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                            retry
                        }
                        .boxed();
                        Some(fut)
                    } else {
                        trace!("retry given up; err={err}");
                        None
                    }
                }
            },
            Self::Never => None,
        }
    }

    fn clone_request(&self, req: &HttpRequest) -> Option<HttpRequest> {
        Some(req.clone())
    }
}

/// Okx HTTP API layer.
pub struct OkxHttpApiLayer<'a, F> {
    host: &'a str,
    key: Option<Key>,
    retry_policy: RetryPolicy<F>,
}

impl<'a, F> OkxHttpApiLayer<'a, F> {
    /// Set key.
    pub fn private(mut self, key: Key) -> Self {
        self.key = Some(key);
        self
    }

    /// Set retry policy.
    pub fn retry<F2>(self, policy: RetryPolicy<F2>) -> OkxHttpApiLayer<'a, F2>
    where
        F2: Clone,
    {
        OkxHttpApiLayer {
            host: self.host,
            retry_policy: policy,
            key: self.key,
        }
    }

    /// Retry on `true`.
    pub fn retry_on<F2>(self, f: F2) -> OkxHttpApiLayer<'a, F2>
    where
        F2: Fn(&ExchangeError) -> bool,
        F2: Send + 'static + Clone,
    {
        self.retry(RetryPolicy::On { f, times: 0 })
    }

    /// Always retry on errors.
    pub fn retry_on_error(self) -> OkxHttpApiLayer<'a, fn(&ExchangeError) -> bool> {
        self.retry(RetryPolicy::On {
            f: |_| true,
            times: 0,
        })
    }
}

impl Default for OkxHttpApiLayer<'static, fn(&ExchangeError) -> bool> {
    fn default() -> Self {
        Self::new("https://www.okx.com")
    }
}

impl<'a> OkxHttpApiLayer<'a, fn(&ExchangeError) -> bool> {
    /// Create a new okx http api layer.
    pub fn new(host: &'a str) -> Self {
        Self {
            host,
            retry_policy: RetryPolicy::Never,
            key: None,
        }
    }
}

impl<'a, S, F> Layer<S> for OkxHttpApiLayer<'a, F>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone,
    S::Future: Send + 'static,
    S::Error: 'static,
    ExchangeError: From<S::Error>,
    F: Fn(&ExchangeError) -> bool,
    F: Send + 'static + Clone,
{
    type Service = Retry<RetryPolicy<F>, OkxHttpApi<S>>;

    fn layer(&self, inner: S) -> Self::Service {
        let svc = OkxHttpApi {
            host: self.host.to_string(),
            http: inner,
            key: self.key.clone(),
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
}

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
                    get.to_request(&self.host, &key)
                } else {
                    Err(ExchangeError::KeyError(anyhow::anyhow!(
                        "key has not been set"
                    )))
                }
            }
        };
        match req {
            Ok(req) => self
                .http
                .call(req)
                .map_err(ExchangeError::from)
                .and_then(|resp| {
                    trace!("http response; status: {:?}", resp.status());
                    hyper::body::to_bytes(resp.into_body())
                        .map_err(|err| ExchangeError::Other(err.into()))
                })
                .and_then(|bytes| {
                    let resp = serde_json::from_slice::<FullHttpResponse>(&bytes)
                        .map_err(|err| ExchangeError::Other(err.into()));

                    futures::future::ready(resp)
                })
                .and_then(|resp| ready(resp.try_into()))
                .boxed(),
            Err(err) => ready(Err(err)).boxed(),
        }
    }
}
