use super::ExcMut;
use crate::{Exc, ExchangeError};
use tower::Service;

/// Request and Response binding.
pub trait Request: Sized {
    /// Response type.
    type Response;
}

/// An adaptor for request.
pub trait Adaptor<R: Request>: Request {
    /// Convert from request.
    fn from_request(req: R) -> Result<Self, ExchangeError>;

    /// Convert into response.
    fn into_response(resp: Self::Response) -> Result<R::Response, ExchangeError>;
}

impl<T, R> Adaptor<R> for T
where
    T: Request,
    R: Request,
    T: TryFrom<R, Error = ExchangeError>,
    T::Response: TryInto<R::Response, Error = ExchangeError>,
{
    fn from_request(req: R) -> Result<Self, ExchangeError>
    where
        Self: Sized,
    {
        Self::try_from(req)
    }

    fn into_response(resp: Self::Response) -> Result<<R as Request>::Response, ExchangeError> {
        resp.try_into()
    }
}

/// Exc service,
pub trait ExcService<R>: Service<R, Response = R::Response, Error = ExchangeError>
where
    R: Request,
{
    /// Create a mutable reference of itself.
    fn as_service_mut(&mut self) -> ExcMut<'_, Self> {
        ExcMut { inner: self }
    }

    #[cfg(feature = "retry")]
    /// Create a retry service.
    fn into_retry(
        self,
        max_duration: std::time::Duration,
    ) -> tower::retry::Retry<crate::retry::Always, Self>
    where
        R: Clone,
        Self: Sized + Clone,
    {
        tower::ServiceBuilder::default()
            .retry(crate::retry::Always::with_max_duration(max_duration))
            .service(self)
    }
}

impl<S, R> ExcService<R> for S
where
    S: Service<R, Response = R::Response, Error = ExchangeError>,
    R: Request,
{
}

/// Service that can be converted into a [`Exc`].
pub trait IntoExc<R>: Service<R, Response = R::Response>
where
    Self::Error: Into<ExchangeError>,
    R: Request,
{
    /// Convert into a [`Exc`].
    fn into_exc(self) -> Exc<Self, R>
    where
        Self: Sized,
    {
        Exc::new(self)
    }
}

impl<S, R> IntoExc<R> for S
where
    S: Service<R, Response = R::Response>,
    S::Error: Into<ExchangeError>,
    R: Request,
{
}
