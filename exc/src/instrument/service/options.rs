use std::time::Duration;

use crate::core::Str;

/// Options of Instruments layer.
#[derive(Debug, Clone)]
pub(crate) struct InstrumentsOptions {
    pub(crate) buffer_bound: usize,
    pub(crate) inst_tags: Vec<Str>,
    pub(crate) fetch_rate_limit: (u64, Duration),
    pub(crate) subscribe_rate_limit: (u64, Duration),
}

impl InstrumentsOptions {
    /// Set instrument tags.
    pub fn tags(mut self, tags: &[&str]) -> Self {
        self.inst_tags = tags.iter().map(Str::new).collect();
        self
    }

    /// Set buffer bound.
    pub fn buffer_bound(mut self, bound: usize) -> Self {
        self.buffer_bound = bound;
        self
    }
}

impl Default for InstrumentsOptions {
    fn default() -> Self {
        Self {
            buffer_bound: 1024,
            inst_tags: vec![Str::new_inline("")],
            fetch_rate_limit: (1, Duration::from_secs(1)),
            subscribe_rate_limit: (1, Duration::from_secs(1)),
        }
    }
}
