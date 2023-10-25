use exc_service::Request;

/// Force reconnect.
#[derive(Debug, Clone, Copy, Default)]
pub struct Reconnect;

impl Request for Reconnect {
    type Response = ();
}
