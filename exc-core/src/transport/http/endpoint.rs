use hyper::client::Builder;

/// Endpoint.
#[derive(Debug, Default)]
pub struct Endpoint {
    #[cfg_attr(
        not(any(feature = "native-tls", feature = "rustls-tls")),
        allow(dead_code)
    )]
    inner: Builder,
}

impl From<Builder> for Endpoint {
    fn from(inner: Builder) -> Self {
        Self { inner }
    }
}

#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
mod https {
    use super::*;
    use crate::transport::http::channel::HttpsChannel;

    impl Endpoint {
        /// Create a https channel.
        pub fn connect_https(&self) -> HttpsChannel {
            cfg_if::cfg_if! {
                if #[cfg(feature = "native-tls")] {
                    let https = hyper_tls::HttpsConnector::new();
                } else if #[cfg(feature = "rustls-tls")] {
                    let https = hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots().https_or_http();
                    #[cfg(not(feature = "http2"))]
                    let https = https.enable_http1();
                    #[cfg(feature = "http2")]
                    let https = https.enable_http2();
                    let https= https.build();
                }
            }
            let client = self.inner.build(https);
            HttpsChannel { inner: client }
        }
    }
}
