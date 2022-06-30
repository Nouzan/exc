/// Request.
pub mod request;

/// Response.
pub mod response;

/// Key.
pub mod key;

/// Adaptations.
pub mod adaptations;

use self::{request::Request, response::Response};

pub use crate::websocket::protocol::frame::Name;

impl exc_core::Request for Request {
    type Response = Response;
}
