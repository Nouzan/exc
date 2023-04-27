#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
pub use https::HttpsChannel;

#[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
/// Https channel.
pub mod https {
    use crate::ExchangeError;
    use futures::{future::BoxFuture, FutureExt, TryFutureExt};
    use http::{Request, Response};
    use hyper::{client::HttpConnector, Body, Client};

    cfg_if::cfg_if! {
        if #[cfg(feature = "native-tls")] {
            use hyper_tls::HttpsConnector;
        } else if #[cfg(feature = "rustls-tls")] {
            use hyper_rustls::HttpsConnector;
        }
    }

    /// Https channel.
    #[derive(Clone)]
    pub struct HttpsChannel {
        pub(crate) inner: Client<HttpsConnector<HttpConnector>>,
    }

    impl tower::Service<Request<Body>> for HttpsChannel {
        type Response = Response<Body>;
        type Error = ExchangeError;
        type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn poll_ready(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx).map_err(ExchangeError::Http)
        }

        fn call(&mut self, req: Request<Body>) -> Self::Future {
            tower::Service::call(&mut self.inner, req)
                .map_err(ExchangeError::Http)
                .boxed()
        }
    }
}
