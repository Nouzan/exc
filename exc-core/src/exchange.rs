pub use exc_make::*;
use exc_types::SubscribeInstruments;
use tower::Service;

/// Make a exchange service.
pub trait MakeExchange: MakeInstruments + MakeTickers + MakeTrading + MakeFetchCandles
where
    <Self as MakeInstruments>::Service: Send + 'static,
    <<Self as MakeInstruments>::Service as Service<SubscribeInstruments>>::Future: Send + 'static,
{
    /// Name of the exchange.
    fn name(&self) -> &str;
}

/// Make a trading service.
pub trait MakeTrading:
    MakePlaceOrder + MakeCancelOrder + MakeCheckOrder + MakeSubscribeOrders
{
}

impl<M> MakeTrading for M where
    M: MakePlaceOrder + MakeCancelOrder + MakeCheckOrder + MakeSubscribeOrders
{
}
