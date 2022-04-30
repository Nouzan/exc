use super::{channel::Channel, connection::Connection};
use crate::error::OkxError;
use http::Uri;
use tower::{buffer::Buffer, timeout::Timeout, ServiceExt};

const DEFAULT_BUFFER_SIZE: usize = 1024;

/// Okx websocket endpoint.
/// Builder for channels.
pub struct Endpoint {
    pub(crate) uri: Uri,
    pub(crate) timeout: Option<std::time::Duration>,
    pub(crate) buffer_size: Option<usize>,
}

impl Endpoint {
    /// Set request timeout. Default to `None`.
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set buffer size. Default to `DEFAULT_BUFFER_SIZE`.
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = Some(buffer_size);
        self
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            uri: Uri::from_static("wss://wsaws.okex.com:8443/ws/v5/public"),
            timeout: None,
            buffer_size: None,
        }
    }
}

impl Endpoint {
    /// Create a okx websocket channel.
    pub fn connect(&self) -> Channel {
        let svc = match self.timeout {
            Some(timeout) => Timeout::new(Connection::new(self), timeout)
                .map_err(OkxError::Layer)
                .boxed(),
            None => Connection::new(self).boxed(),
        };
        let buffer_size = self.buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);
        let svc = Buffer::new(svc, buffer_size);
        Channel { svc }
    }
}
