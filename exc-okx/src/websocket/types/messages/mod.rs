/// Event.
pub mod event;

/// Request.
pub mod request;

// /// Response.
// pub mod response;

use exc_core::Str;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;

/// Okx arguments.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Args(pub BTreeMap<Str, Str>);

impl Args {
    /// Args for tickers subscription.
    pub fn subscribe_tickers(inst: &str) -> Self {
        Args(BTreeMap::from([
            (Str::new_inline("channel"), Str::new_inline("tickers")),
            (Str::new_inline("instId"), Str::new(inst)),
        ]))
    }

    /// Args for orders subscription.
    pub fn subscribe_orders(inst: &str) -> Self {
        Args(BTreeMap::from([
            (Str::new_inline("channel"), Str::new_inline("orders")),
            (Str::new_inline("instType"), Str::new_inline("ANY")),
            (Str::new_inline("instId"), Str::new(inst)),
        ]))
    }

    /// Args for trades subscription.
    pub fn subscribe_trades(inst: &str) -> Self {
        Args(BTreeMap::from([
            (Str::new_inline("channel"), Str::new_inline("trades")),
            (Str::new_inline("instId"), Str::new(inst)),
        ]))
    }

    /// Args for bid/ask subscription.
    pub fn subscribe_bid_ask(inst: &str) -> Self {
        Args(BTreeMap::from([
            (Str::new_inline("channel"), Str::new_inline("bbo-tbt")),
            (Str::new_inline("instId"), Str::new(inst)),
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
