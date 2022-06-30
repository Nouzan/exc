use std::{
    str::FromStr,
    task::{Context, Poll},
};

use exc_core::transport::websocket::{connector::WsConnector, WsStream};
use futures::{future::BoxFuture, FutureExt};
use http::Uri;
use tower::Service;

use crate::types::Name;

use super::error::WsError;

#[derive(Debug, Clone)]
pub(crate) struct BinanceWsTarget {
    pub(crate) host: String,
    pub(crate) name: Name,
}

impl BinanceWsTarget {
    fn to_uri(&self) -> Result<Uri, WsError> {
        let uri = Uri::from_str(format!("{}/ws/{}", self.host, self.name).as_str())?;
        Ok(uri)
    }
}

#[derive(Default)]
pub(crate) struct BinanceWsConnect {
    inner: WsConnector,
}

impl Service<BinanceWsTarget> for BinanceWsConnect {
    type Response = WsStream;
    type Error = WsError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(WsError::from)
    }

    fn call(&mut self, req: BinanceWsTarget) -> Self::Future {
        let res = req.to_uri().map(|uri| self.inner.call(uri));
        async move { Ok(res?.await?) }.boxed()
    }
}
