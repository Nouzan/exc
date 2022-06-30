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

#[derive(Clone)]
pub(crate) enum TargetKind {
    Name(Name),
    ListenKey(Http),
}

#[derive(Clone)]
pub(crate) struct BinanceWsTarget {
    pub(crate) host: String,
    pub(crate) kind: TargetKind,
}

impl BinanceWsTarget {
    async fn into_uri(self) -> Result<Uri, WsError> {
        match self.kind {
            TargetKind::Name(name) => Ok(Uri::from_str(
                format!("{}/ws/{}", self.host, name).as_str(),
            )?),
            TargetKind::ListenKey(http) => {
                let listen_key = http
                    .clone()
                    .oneshot(RestRequest::with_payload(CurrentListenKey))
                    .await?
                    .into_response::<ListenKey>()?;
                Ok(Uri::from_str(
                    format!("{}/ws/{listen_key}", self.host).as_str(),
                )?)
            }
        }
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
