use exc_make::{MakeInstruments, MakeTickers};

/// Make a exchange service.
pub trait MakeExchange: MakeInstruments + MakeTickers {
    /// Name of the exchange.
    fn name(&self) -> &str;
}
