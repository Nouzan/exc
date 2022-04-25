/// Ticker.
pub mod ticker;

/// Data trait.
pub trait Data {}

impl<T> Data for T {}

/// Box Data.
pub type BoxData = Box<dyn Data>;
