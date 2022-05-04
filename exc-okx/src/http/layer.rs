use super::types::{request::HttpRequest, response::HttpResponse};
use exc::ExchangeError;
use futures::{future::BoxFuture, FutureExt};
use http::{Request, Response};
use hyper::Body;
use tower::Service;

/// Okx HTTP API Service.
pub struct OkxHttpApi<S> {
    host: String,
    http: S,
}

impl<S> Service<HttpRequest> for OkxHttpApi<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
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
            Ok(req) => {
                let res = self.http.call(req);
                todo!()
            }
            Err(err) => futures::future::ready(Err(err)).boxed(),
        }
    }
}
