use super::adapt::{Adapt, AdaptLayer, AdaptService};
use crate::{Exc, ExchangeError};
use tower::{util::MapErr, Layer, Service};

/// Request and Response binding.
pub trait Request: Sized {
    /// Response type.
    type Response;
}

/// An alias of [`Service`] that requires the input type to be a [`Request`],
/// and the error type to be [`ExchangeError`].
///
/// Basically, [`Request`] is just indicating that the input type has a fixed response type.
pub trait ExcService<R>: Service<R, Response = R::Response, Error = ExchangeError>
where
    R: Request,
{
    /// Apply a layer of which the result service is still a [`ExcService`].
    fn apply<L, R2>(self, layer: &L) -> L::Service
    where
        Self: Sized,
        R2: Request,
        L: Layer<Self>,
        L::Service: ExcService<R2>,
    {
        layer.layer(self)
    }

    /// Adapt the request type to the given.
    fn adapt<R2>(self) -> Adapt<Self, R, R2>
    where
        Self: Sized,
        R2: Request,
        Self: AdaptService<R, R2>,
    {
        self.apply(&AdaptLayer::default())
    }
}

impl<S, R> ExcService<R> for S
where
    S: Service<R, Response = R::Response, Error = ExchangeError>,
    R: Request,
{
}

type MapErrFn<E> = fn(E) -> ExchangeError;

/// Service that can be converted into a [`Exc`].
pub trait IntoExc<R>: Service<R, Response = R::Response>
where
    Self::Error: Into<ExchangeError>,
    R: Request,
{
    /// Convert into a [`Exc`].
    fn into_exc(self) -> Exc<MapErr<Self, MapErrFn<Self::Error>>, R>
    where
        Self: Sized,
    {
        Exc::new(MapErr::new(self, Self::Error::into))
    }
}

impl<S, R> IntoExc<R> for S
where
    S: Service<R, Response = R::Response>,
    S::Error: Into<ExchangeError>,
    R: Request,
{
}
