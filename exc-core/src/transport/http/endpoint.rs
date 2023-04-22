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
        cfg_if::cfg_if! {
            if #[cfg(feature = "native-tls")] {
                let https = hyper_tls::HttpsConnector::new();
            } else if #[cfg(feature = "rustls-tls")] {
                let https = hyper_rustls::HttpsConnectorBuilder::new()
                    .with_webpki_roots()
                    .https_or_http()
                    .enable_http1()
                    .build();
            } else {
                compile_error!{"Not support TLS"}
            }
        }
        let client = self.inner.build(https);
        HttpsChannel { inner: client }
    }
}

impl From<Builder> for Endpoint {
    fn from(inner: Builder) -> Self {
        Self { inner }
    }
}
