use crate::{error::ExchangeError, request::Request, response::Response};
use std::{
    future::Future,
    task::{Context, Poll},
};
use tower::Service;

/// Exchange Service.
pub trait ExchangeService<ReqData> {
    /// Response.
    type RespData;
    /// Error type.
    type Error: Into<ExchangeError>;
    /// Future.
    type Future: Future<Output = Result<Response<Self::RespData>, Self::Error>>;

    /// Poll ready.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Call.
    fn call(&mut self, request: Request<ReqData>) -> Self::Future;
}

impl<T, ReqData, RespData> ExchangeService<ReqData> for T
where
    T: Service<Request<ReqData>, Response = Response<RespData>>,
    T::Error: Into<ExchangeError>,
{
    type RespData = RespData;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(self, cx)
    }

    fn call(&mut self, request: Request<ReqData>) -> Self::Future {
        Service::call(self, request)
    }
}
