use super::types::{request::HttpRequest, response::HttpResponse};
use exc::ExchangeError;
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use http::{Request, Response};
use hyper::Body;
use tower::{Layer, Service};

/// Okx HTTP API layer.
pub struct OkxHttpApiLayer<'a> {
    host: &'a str,
}

impl Default for OkxHttpApiLayer<'static> {
    fn default() -> Self {
        Self::new("https://www.okx.com")
    }
}

impl<'a> OkxHttpApiLayer<'a> {
    /// Create a new okx http api layer.
    pub fn new(host: &'a str) -> Self {
        Self { host }
    }
}

impl<'a, S> Layer<S> for OkxHttpApiLayer<'a> {
    type Service = OkxHttpApi<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OkxHttpApi {
            host: self.host.to_string(),
            http: inner,
        }
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
                .boxed(),
            Err(err) => futures::future::ready(Err(err)).boxed(),
        }
    }
}
