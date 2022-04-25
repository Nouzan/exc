/// Event.
pub mod event;

/// Request.
pub mod request;

/// Response.
pub mod response;

/// Envelope.
pub mod envelope;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Okx arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Args(pub HashMap<String, String>);

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
