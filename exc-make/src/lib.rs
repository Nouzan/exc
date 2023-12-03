#![deny(missing_docs)]

//! `exc-make`: Define what an exchange is with [`MakeService`](tower_make::MakeService)s.

/// Make a service to subscribe instruments.
pub mod instruments;

/// Make a service to subscribe tickers.
pub mod tickers;

/// Make a service to fetch candles.
pub mod candles;

/// Make a service to place orders.
pub mod place;

/// Make a service to cancel orders.
pub mod cancel;

/// Make a service to check orders.
pub mod check;

/// Make a service to subscribe order updates.
pub mod orders;

pub use self::{
    cancel::{MakeCancelOrder, MakeCancelOrderOptions},
    candles::{MakeFetchCandles, MakeFetchCandlesOptions},
    check::{MakeCheckOrder, MakeCheckOrderOptions},
    instruments::{MakeInstruments, MakeInstrumentsOptions},
    orders::{MakeSubscribeOrders, MakeSubscribeOrdersOptions},
    place::{MakePlaceOrder, MakePlaceOrderOptions},
    tickers::{MakeTickers, MakeTickersOptions},
};

/// Define `AsService`s.
#[macro_export(local_inner_macros)]
macro_rules! create_as_service {
    ($make_trait:ident, $options_type:ident, $method:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug)]
        pub struct AsService<'a, M>
        where
            M: $make_trait,
        {
            make: &'a mut M,
        }

        impl<M> tower_service::Service<$options_type> for AsService<'_, M>
        where
            M: $make_trait,
        {
            type Response = M::Service;
            type Error = ExchangeError;
            type Future = M::Future;

            #[inline]
            fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                self.make.poll_ready(cx)
            }

            #[inline]
            fn call(&mut self, options: $options_type) -> Self::Future {
                self.make.$method(options)
            }
        }
    };
}
