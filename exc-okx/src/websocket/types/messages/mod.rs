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
