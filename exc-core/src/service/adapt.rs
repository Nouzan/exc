use std::{fmt, future::Future, marker::PhantomData};

use futures::TryFuture;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use crate::{ExcService, ExchangeError, Request};

/// An adaptor for request.
pub trait Adaptor<R: Request>: Request {
    /// Convert from request.
    fn from_request(req: R) -> Result<Self, ExchangeError>;

    /// Convert into response.
    fn into_response(resp: Self::Response) -> Result<R::Response, ExchangeError>;
}

impl<T, R, E> Adaptor<R> for T
where
    T: Request,
    R: Request,
    T: TryFrom<R, Error = E>,
    T::Response: TryInto<R::Response, Error = E>,
    ExchangeError: From<E>,
{
    fn from_request(req: R) -> Result<Self, ExchangeError>
    where
        Self: Sized,
    {
        Ok(Self::try_from(req)?)
    }

    fn into_response(resp: Self::Response) -> Result<<R as Request>::Response, ExchangeError> {
        Ok(resp.try_into()?)
    }
}

/// Layer for creating [`Adapted`].
#[derive(Debug)]
pub struct AdaptLayer<Req, R>(PhantomData<fn() -> (Req, R)>);

impl<Req, R> Default for AdaptLayer<Req, R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S, Req, R> Layer<S> for AdaptLayer<Req, R> {
    type Service = Adapt<S, Req, R>;

    fn layer(&self, inner: S) -> Self::Service {
        Adapt(inner, PhantomData)
    }
}

/// Service that can handle request [`R`] with its inner request [`Req`].
pub trait AdaptService<Req, R>: ExcService<Req>
where
    Req: Request,
    R: Request,
{
    /// Future returned by [`AdaptService::into_response`].
    type AdaptedResponse: Future<Output = Result<R::Response, ExchangeError>>;

    /// Adapt the request.
    fn from_request(&mut self, req: R) -> Result<Req, ExchangeError>;

    /// Adapt the response future
    fn into_response(&mut self, res: Self::Future) -> Self::AdaptedResponse;
}

pin_project! {
    /// Future for [`AdaptService`] implementation.
    #[derive(Debug)]
    pub struct AndThen<Fut, F> {
        #[pin]
        fut: Fut,
        f: Option<F>,
    }
}

impl<Fut, F> AndThen<Fut, F>
where
    Fut: TryFuture<Error = ExchangeError>,
{
    /// Create a new [`AndThen`] future.
    pub fn new(fut: Fut, f: F) -> Self {
        Self { fut, f: Some(f) }
    }
}

impl<Fut, F, T> Future for AndThen<Fut, F>
where
    Fut: TryFuture<Error = ExchangeError>,
    F: FnOnce(Fut::Ok) -> Result<T, ExchangeError>,
{
    type Output = Result<T, ExchangeError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        match this.fut.try_poll(cx) {
            std::task::Poll::Ready(Ok(ok)) => match this.f.take() {
                Some(f) => std::task::Poll::Ready((f)(ok)),
                None => return std::task::Poll::Pending,
            },
            std::task::Poll::Ready(Err(err)) => std::task::Poll::Ready(Err(err)),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl<C, Req, R> AdaptService<Req, R> for C
where
    Req: Request,
    R: Request,
    Req: Adaptor<R>,
    C: ExcService<Req>,
{
    type AdaptedResponse =
        AndThen<Self::Future, fn(Req::Response) -> Result<R::Response, ExchangeError>>;

    fn from_request(&mut self, req: R) -> Result<Req, ExchangeError> {
        Req::from_request(req)
    }

    fn into_response(&mut self, res: Self::Future) -> Self::AdaptedResponse {
        AndThen::new(res, Req::into_response)
    }
}

/// Adapt Service Wrapper.
pub struct Adapt<C, Req, R>(C, PhantomData<fn() -> (Req, R)>);

impl<C, Req, R> fmt::Debug for Adapt<C, Req, R>
where
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Adapt")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl<C, Req, R> Clone for Adapt<C, Req, R>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<C, Req, R> Copy for Adapt<C, Req, R> where C: Copy {}

pin_project! {
    /// Future returned by [`Adapt`].
    #[allow(missing_docs)]
    #[project = AdaptProj]
    #[derive(Debug)]
    pub enum AdaptFuture<Fut> {
        /// From request error.
        FromRequestError {
            err: Option<ExchangeError>,
        },
        /// Into response.
        IntoResponse {
            #[pin]
            fut: Fut,
        }
    }
}

impl<Fut> AdaptFuture<Fut> {
    /// Create a new [`AdaptFuture::FromRequestError`].
    pub fn from_request_error(err: ExchangeError) -> Self {
        Self::FromRequestError { err: Some(err) }
    }

    /// Create a new [`AdaptFuture::IntoResponse`].
    pub fn into_response(fut: Fut) -> Self {
        Self::IntoResponse { fut }
    }
}

impl<Fut> Future for AdaptFuture<Fut>
where
    Fut: TryFuture<Error = ExchangeError>,
{
    type Output = Result<Fut::Ok, ExchangeError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.as_mut().project() {
            AdaptProj::FromRequestError { err } => match err.take() {
                Some(err) => std::task::Poll::Ready(Err(err)),
                None => std::task::Poll::Pending,
            },
            AdaptProj::IntoResponse { fut, .. } => fut.try_poll(cx),
        }
    }
}

impl<C, Req, R> Service<R> for Adapt<C, Req, R>
where
    C: AdaptService<Req, R>,
    Req: Request,
    R: Request,
{
    type Response = R::Response;

    type Error = ExchangeError;

    type Future = AdaptFuture<C::AdaptedResponse>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: R) -> Self::Future {
        let req = match self.0.from_request(req) {
            Ok(req) => req,
            Err(err) => return AdaptFuture::from_request_error(err),
        };
        let res = self.0.call(req);
        AdaptFuture::into_response(self.0.into_response(res))
    }
}
