use crate::ExchangeError;

/// Ticker.
pub mod ticker;

/// Candle.
pub mod candle;

/// Instrument.
pub mod instrument;

/// Key.
pub mod key;

/// Trading.
pub mod trading;

pub use candle::{
    Candle, CandleStream, Period, PeriodKind, QueryCandles, QueryFirstCandles, QueryLastCandles,
};

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
