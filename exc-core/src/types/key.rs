use std::{fmt, str::FromStr, sync::Arc};

/// Key.
#[derive(Debug)]
pub struct Key {
    inner: Arc<String>,
}

impl FromStr for Key {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: Arc::new(s.to_string()),
        })
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Key {
    /// Convert to [`&str`]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}
