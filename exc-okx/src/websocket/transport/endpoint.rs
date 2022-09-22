use super::{channel::Channel, connection::Connection};
use crate::{error::OkxError, key::Key};
use http::Uri;
use std::time::Duration;
use tower::{buffer::Buffer, timeout::Timeout, ServiceExt};

const DEFAULT_BUFFER_SIZE: usize = 1024;
const DEFAULT_PING_TIMEOUT: Duration = Duration::from_secs(15);

/// Okx websocket endpoint.
/// Builder for channels.
pub struct Endpoint {
    pub(crate) testing: bool,
    pub(crate) aws: bool,
    pub(crate) request_timeout: Option<Duration>,
    pub(crate) connection_timeout: Option<Duration>,
    pub(crate) ping_timeout: Duration,
    pub(crate) buffer_size: Option<usize>,
    pub(crate) login: Option<Key>,
}

impl Endpoint {
    /// Set request timeout. Default to `None`.
    pub fn request_timeout(mut self, duration: Duration) -> Self {
        self.request_timeout = Some(duration);
        self
    }

    /// Set request timeout. Default to `None`.
    pub fn connection_timeout(mut self, duration: Duration) -> Self {
        self.connection_timeout = Some(duration);
        self
    }

    /// Set buffer size. Default to `DEFAULT_BUFFER_SIZE`.
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = Some(buffer_size);
        self
    }

    /// Set ping timeout. Default to `DEFAULT_PING_TIMEOUT`.
    pub fn ping_timeout(mut self, duration: Duration) -> Self {
        self.ping_timeout = duration;
        self
    }

    /// Switch to testing environment.
    pub fn testing(mut self, enable: bool) -> Self {
        self.testing = enable;
        self
    }

    /// Switch to AWS endpoint.
    pub fn aws(mut self, enable: bool) -> Self {
        self.aws = enable;
        self
    }

    /// Switch to private channel.
    pub fn private(mut self, key: Key) -> Self {
        self.login = Some(key);
        self
    }

    /// Create a okx websocket channel.
    pub fn connect(&self) -> Channel {
        let svc = match self.request_timeout {
            Some(timeout) => Timeout::new(Connection::new(self), timeout)
                .map_err(OkxError::Layer)
                .boxed(),
            None => Connection::new(self).boxed(),
        };
        let buffer_size = self.buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);
        let (svc, worker) = Buffer::pair(svc, buffer_size);
        let handle = tokio::spawn(async move {
            worker.await;
            debug!("buffer worker is dead");
        });
        tokio::spawn(async move {
            if let Err(err) = handle.await {
                error!("buffer worker task error: {err}");
            }
        });

        Channel { svc }
    }

    /// Get current uri.
    pub fn uri(&self) -> Uri {
        match (self.login.is_some(), self.testing, self.aws) {
            (true, true, _) => {
                Uri::from_static("wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999")
            }
            (false, true, _) => {
                Uri::from_static("wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999")
            }
            (true, false, true) => Uri::from_static("wss://wsaws.okx.com:8443/ws/v5/private"),
            (false, false, true) => Uri::from_static("wss://wsaws.okx.com:8443/ws/v5/public"),
            (true, false, false) => Uri::from_static("wss://ws.okx.com:8443/ws/v5/private"),
            (false, false, false) => Uri::from_static("wss://ws.okx.com:8443/ws/v5/public"),
        }
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            aws: false,
            testing: false,
            request_timeout: None,
            connection_timeout: None,
            buffer_size: None,
            ping_timeout: DEFAULT_PING_TIMEOUT,
            login: None,
        }
    }
}
