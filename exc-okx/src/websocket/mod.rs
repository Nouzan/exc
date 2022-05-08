use exc::types::Request as ExcRequest;

/// Okx websocket transport.
pub mod transport;

/// Types definitions for okx websocket api.
pub mod types;

/// Adaptations.
pub mod adaptations;

pub use transport::endpoint::Endpoint;
pub use types::{request::Request, response::Response};

impl ExcRequest for Request {
    type Response = Response;
}
