use std::{
    str::FromStr,
    task::{Context, Poll},
};

use exc_core::transport::{
    http::channel::HttpsChannel,
    websocket::{connector::WsConnector, WsStream},
};
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use http::Uri;
use tower::{Service, ServiceExt};

use crate::{
    http::{
        request::{CurrentListenKey, RestRequest},
        response::ListenKey,
        BinanceRestApi,
    },
    types::Name,
};

use super::error::WsError;

pub(crate) type Http = BinanceRestApi<HttpsChannel>;

#[derive(Debug, Clone, Copy)]
pub enum BinanceWsHost {
    UsdMarginFutures,
    UsdMarginFuturesPrivate,
}

impl BinanceWsHost {
    fn as_str(&self) -> &'static str {
        match self {
            Self::UsdMarginFutures => "wss://fstream.binance.com",
            Self::UsdMarginFuturesPrivate => "wss://fstream-auth.binance.com",
        }
    }

    pub(crate) fn private(&mut self) {
        match *self {
            Self::UsdMarginFutures => *self = Self::UsdMarginFuturesPrivate,
            _ => {}
        }
    }
}

#[derive(Clone)]
pub(crate) struct BinanceWsTarget {
    pub(crate) host: BinanceWsHost,
    pub(crate) name: Name,
    pub(crate) key_provider: Option<Http>,
}

impl BinanceWsTarget {
    async fn into_uri(self) -> Result<Uri, WsError> {
        let mut uri = format!("{}/stream?streams={}", self.host.as_str(), self.name);
        if let Some(provider) = self.key_provider {
            let listen_key = provider
                .oneshot(RestRequest::with_payload(CurrentListenKey))
                .await?
                .into_response::<ListenKey>()?;
            uri.push_str("/");
            uri.push_str(listen_key.as_str());
            uri.push_str("&listenKey=");
            uri.push_str(listen_key.as_str());
        }
        tracing::trace!("ws uri={uri}");
        Ok(Uri::from_str(uri.as_str())?)
    }
}

#[derive(Default)]
pub(crate) struct BinanceWsConnect;

impl Service<BinanceWsTarget> for BinanceWsConnect {
    type Response = WsStream;
    type Error = WsError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: BinanceWsTarget) -> Self::Future {
        let connect = WsConnector::default();
        let res = req
            .into_uri()
            .and_then(|uri| connect.oneshot(uri).map_err(WsError::from));
        async move { Ok(res.await?) }.boxed()
    }
}
