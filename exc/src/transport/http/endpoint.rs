use hyper::client::Builder;

use super::channel::HttpsChannel;

/// Endpoint.
#[derive(Debug, Default)]
pub struct Endpoint {
    inner: Builder,
}

impl Endpoint {
    /// Create a https channel.
    pub fn connect_https(&self) -> HttpsChannel {
        let https = hyper_tls::HttpsConnector::new();
        let client = self.inner.build(https);
        HttpsChannel { inner: client }
    }
}

impl From<Builder> for Endpoint {
    fn from(inner: Builder) -> Self {
        Self { inner }
    }
}
