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

const CHANNEL: Str = Str::new_inline("channel");
const INST_ID: Str = Str::new_inline("instId");

impl Args {
    /// Args for instruments subscription.
    pub fn subscribe_instruments(inst_type: &str) -> Self {
        Args(BTreeMap::from([
            (Str::new_inline("channel"), Str::new_inline("instruments")),
            (Str::new_inline("instType"), Str::new(inst_type)),
        ]))
    }

    /// Args for tickers subscription.
    pub fn subscribe_tickers(inst: &str) -> Self {
        Args(BTreeMap::from([
            (CHANNEL, Str::new_inline("tickers")),
            (INST_ID, Str::new(inst)),
        ]))
    }

    /// Args for orders subscription.
    pub fn subscribe_orders(inst: &str) -> Self {
        Args(BTreeMap::from([
            (CHANNEL, Str::new_inline("orders")),
            (Str::new_inline("instType"), Str::new_inline("ANY")),
            (INST_ID, Str::new(inst)),
        ]))
    }

    /// Args for trades subscription.
    pub fn subscribe_trades(inst: &str) -> Self {
        Args(BTreeMap::from([
            (CHANNEL, Str::new_inline("trades")),
            (INST_ID, Str::new(inst)),
        ]))
    }

    /// Args for bid/ask subscription.
    pub fn subscribe_bid_ask(inst: &str) -> Self {
        Args(BTreeMap::from([
            (CHANNEL, Str::new_inline("bbo-tbt")),
            (INST_ID, Str::new(inst)),
        ]))
    }

    /// Args for option summary subscription.
    pub fn subscribe_option_summary(inst_family: &str) -> Self {
        Args(BTreeMap::from([
            (CHANNEL, Str::new_inline("opt-summary")),
            (Str::new_inline("instFamily"), Str::new(inst_family)),
        ]))
    }

    /// Args for channel subscription.
    pub fn subscribe_channel<'a>(
        channel: &str,
        args: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        let mut map = BTreeMap::new();
        map.insert(CHANNEL, Str::new(channel));
        for (k, v) in args {
            map.insert(Str::new(k), Str::new(v));
        }
        Args(map)
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
