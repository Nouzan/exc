use crate::ExchangeError;

use super::types::Request;
use tower::Service;

/// Subscribe tickers.
pub mod subscribe_tickers;

/// Fetch candles.
pub mod fetch_candles;

/// Exchange Service,
/// an alias of [`tower::Service`].
pub trait ExchangeService<R>: Service<R, Response = R::Response, Error = ExchangeError>
where
    R: Request,
{
}

impl<S, R> ExchangeService<R> for S
where
    S: Service<R, Response = R::Response, Error = ExchangeError>,
    R: Request,
{
}
