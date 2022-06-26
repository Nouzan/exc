/// Request.
pub mod request;

/// Response.
pub mod response;

/// Adaptations.
pub mod adaptations;

use self::{request::Request, response::Response};

impl exc::types::Request for Request {
    type Response = Response;
}
