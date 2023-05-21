use crate::Request;

/// Force reconnect.
#[derive(Debug, Clone, Copy, Default)]
pub struct Reconnect;

impl Request for Reconnect {
    type Response = ();
}
