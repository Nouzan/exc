use crate::core::Str;

/// Options of market layer.
#[derive(Debug, Clone)]
pub struct MarketOptions {
    pub(crate) buffer_bound: usize,
    pub(crate) inst_tags: Vec<Str>,
}

impl MarketOptions {
    /// Set instrument tags.
    pub fn tags(mut self, tags: &[&str]) -> Self {
        self.inst_tags = tags.iter().map(Str::new).collect();
        self
    }
}

impl Default for MarketOptions {
    fn default() -> Self {
        Self {
            buffer_bound: 1024,
            inst_tags: vec![Str::new_inline("")],
        }
    }
}
