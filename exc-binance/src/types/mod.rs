/// Request.
pub mod request;

/// Response.
pub mod response;

/// Adaptations.
pub mod adaptations;

use self::{request::Request, response::Response};

pub use crate::websocket::protocol::frame::Name;

impl exc_core::types::Request for Request {
    type Response = Response;
}
