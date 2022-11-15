/// Event.
pub mod event;

/// Request.
pub mod request;

// /// Response.
// pub mod response;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;

/// Okx arguments.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Args(pub BTreeMap<String, String>);

impl Args {
    /// Args for tickers subscription.
    pub fn subscribe_tickers(inst: &str) -> Self {
        Args(BTreeMap::from([
            ("channel".to_string(), "tickers".to_string()),
            ("instId".to_string(), inst.to_string()),
        ]))
    }

    /// Args for orders subscription.
    pub fn subscribe_orders(inst: &str) -> Self {
        Args(BTreeMap::from([
            ("channel".to_string(), "orders".to_string()),
            ("instType".to_string(), "ANY".to_string()),
            ("instId".to_string(), inst.to_string()),
        ]))
    }

    pub(crate) fn to_tag(&self) -> String {
        const IGNORE_KEYS: [&str; 1] = ["uid"];
        let mut tag = String::from("sub");
        for (key, value) in self.0.iter() {
            if IGNORE_KEYS.contains(&key.as_str()) {
                continue;
            }
            tag.push(':');
            tag.push_str(value.as_str());
        }
        tag
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl Hash for Args {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (k, v) in self.0.iter() {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl PartialEq for Args {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Args {}
