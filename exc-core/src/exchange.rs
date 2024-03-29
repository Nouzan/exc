pub use exc_make::*;

/// Make a exchange service.
pub trait MakeExchange: MakeInstruments + MakeTickers + MakeTrading + MakeFetchCandles {
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
