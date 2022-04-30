/// Subscribe tickers.
#[derive(Debug, Clone)]
pub struct SubscribeTickers {
    /// Instrument.
    pub instrument: String,
}

impl SubscribeTickers {
    /// Create a new [`SubscribeTickers`]
    pub fn new(inst: &str) -> Self {
	Self {
	    instrument: inst.to_string(),
	}
    }
}
