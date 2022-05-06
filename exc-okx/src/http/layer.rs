use super::types::{request::HttpRequest, response::HttpResponse};
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
pub enum RetryPolicy {
    /// Always.
    Always(usize),
    /// Never.
    Never,
}

impl Policy<HttpRequest, HttpResponse, ExchangeError> for RetryPolicy {
    type Future = BoxFuture<'static, Self>;

    fn retry(
        &self,
        _req: &HttpRequest,
        result: Result<&HttpResponse, &ExchangeError>,
    ) -> Option<Self::Future> {
        match self {
            Self::Always(times) => match result {
                Ok(_) => None,
                Err(err) => {
                    let times = *times;
                    let secs = (1 << times).min(128);
                    error!("request error: {err}; retry in {secs}s...");
                    let retry = Self::Always(times + 1);
                    let fut = async move {
                        tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                        retry
                    }
                    .boxed();
                    Some(fut)
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
pub struct OkxHttpApiLayer<'a> {
    host: &'a str,
    retry_policy: RetryPolicy,
}

impl<'a> OkxHttpApiLayer<'a> {
    /// Set retry policy.
    pub fn retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Alwasy retry on error.
    pub fn retry_on_error(self) -> Self {
        self.retry(RetryPolicy::Always(0))
    }
}

impl Default for OkxHttpApiLayer<'static> {
    fn default() -> Self {
        Self::new("https://www.okx.com")
    }
}

impl<'a> OkxHttpApiLayer<'a> {
    /// Create a new okx http api layer.
    pub fn new(host: &'a str) -> Self {
        Self {
            host,
            retry_policy: RetryPolicy::Never,
        }
    }
}

impl<'a, S> Layer<S> for OkxHttpApiLayer<'a>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone,
    S::Future: Send + 'static,
    S::Error: 'static,
    ExchangeError: From<S::Error>,
{
    type Service = Retry<RetryPolicy, OkxHttpApi<S>>;

    fn layer(&self, inner: S) -> Self::Service {
        let svc = OkxHttpApi {
            host: self.host.to_string(),
            http: inner,
        };
        ServiceBuilder::default()
            .retry(self.retry_policy)
            .service(svc)
    }
}

/// Okx HTTP API Service.
#[derive(Clone)]
pub struct OkxHttpApi<S> {
    host: String,
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
                    let resp = serde_json::from_slice::<HttpResponse>(&bytes)
                        .map_err(|err| ExchangeError::Other(err.into()));

                    futures::future::ready(resp)
                })
                .and_then(|resp| match resp.code.as_str() {
                    "0" => ready(Ok(resp)),
                    _ => ready(Err(ExchangeError::Other(anyhow::anyhow!(
                        "code={} msg={}",
                        resp.code,
                        resp.msg
                    )))),
                })
                .boxed(),
            Err(err) => ready(Err(err)).boxed(),
        }
    }
}
