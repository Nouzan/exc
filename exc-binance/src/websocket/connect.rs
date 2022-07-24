use std::{
    collections::HashSet,
    str::FromStr,
    task::{Context, Poll},
    time::Duration,
};

use exc_core::transport::{http::channel::HttpsChannel, websocket::connector::WsConnector};
use futures::{future::BoxFuture, Future, FutureExt, TryFutureExt};
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

use super::{error::WsError, protocol::WsClient};

pub(crate) type Http = BinanceRestApi<HttpsChannel>;

#[derive(Debug, Clone, Copy)]
pub enum BinanceWsHost {
    UsdMarginFutures,
    UsdMarginFuturesPrivate,
    Spot,
    SpotPrivate,
}

impl BinanceWsHost {
    fn as_str(&self) -> &'static str {
        match self {
            Self::UsdMarginFutures => "wss://fstream.binance.com",
            Self::UsdMarginFuturesPrivate => "wss://fstream-auth.binance.com",
            Self::Spot | Self::SpotPrivate => "wss://stream.binance.com:9443",
        }
    }

    pub(crate) fn private(&mut self) {
        match *self {
            Self::UsdMarginFutures => *self = Self::UsdMarginFuturesPrivate,
            Self::Spot => *self = Self::SpotPrivate,
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

const RETRY: usize = 5;
const INTERVAL: Duration = Duration::from_secs(60 * 5);

impl BinanceWsTarget {
    async fn fresh_key_worker(
        mut provider: Http,
        key: ListenKey,
        retry: Option<usize>,
        interval: Option<Duration>,
    ) {
        let mut failed = false;
        loop {
            tokio::time::sleep(interval.unwrap_or(INTERVAL)).await;
            for _ in 0..retry.unwrap_or(RETRY) {
                match (&mut provider)
                    .oneshot(RestRequest::with_payload(CurrentListenKey))
                    .await
                {
                    Ok(current) => {
                        failed = false;
                        match current.into_response::<ListenKey>() {
                            Ok(current) => {
                                if current.as_str() != key.as_str() {
                                    tracing::error!("fresh worker; listen key is expired");
                                    return;
                                } else {
                                    tracing::trace!("fresh worker; listen key refreshed");
                                }
                            }
                            Err(err) => {
                                tracing::error!(
                                    "fresh worker; parse refresh response error: {err}"
                                );
                                return;
                            }
                        }
                        break;
                    }
                    Err(err) => {
                        tracing::error!("fresh worker; refresh listen key error: {err}");
                        failed = true;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            if failed {
                break;
            }
        }
        tracing::error!("fresh worker; refresh failed");
    }

    async fn into_uri(
        self,
        retry: Option<usize>,
        interval: Option<Duration>,
    ) -> Result<(Uri, Option<impl Future<Output = ()>>), WsError> {
        let mut uri = format!("{}/stream?streams={}", self.host.as_str(), self.name);
        let mut worker = None;
        if let Some(mut provider) = self.key_provider {
            let listen_key = (&mut provider)
                .oneshot(RestRequest::with_payload(CurrentListenKey))
                .await?
                .into_response::<ListenKey>()?;
            tracing::debug!("got listen key");
            uri.push_str("/");
            uri.push_str(listen_key.as_str());
            if matches!(self.host, BinanceWsHost::UsdMarginFuturesPrivate) {
                uri.push_str("&listenKey=");
                uri.push_str(listen_key.as_str());
            }
            worker = Some(Self::fresh_key_worker(
                provider, listen_key, retry, interval,
            ));
        }
        tracing::trace!("ws uri={uri}");
        Ok((Uri::from_str(uri.as_str())?, worker))
    }
}

pub(crate) struct BinanceWsConnect {
    pub(crate) main_stream: HashSet<Name>,
    pub(crate) keep_alive_timeout: Duration,
    pub(crate) default_stream_timeout: Duration,
    pub(crate) retry: Option<usize>,
    pub(crate) interval: Option<Duration>,
}

impl Service<BinanceWsTarget> for BinanceWsConnect {
    type Response = WsClient;
    type Error = WsError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: BinanceWsTarget) -> Self::Future {
        let connect = WsConnector::default();
        let res = req
            .into_uri(self.retry, self.interval)
            .and_then(|(uri, worker)| {
                connect
                    .oneshot(uri)
                    .map_ok(|ws| (ws, worker.map(|f| f.boxed())))
                    .map_err(WsError::from)
            });
        let main_stream = self.main_stream.clone();
        let keep_alive_timeout = self.keep_alive_timeout.clone();
        let default_stream_timeout = self.default_stream_timeout.clone();
        async move {
            let (ws, worker) = res.await?;
            WsClient::with_websocket(
                ws,
                main_stream,
                keep_alive_timeout,
                default_stream_timeout,
                worker,
            )
        }
        .boxed()
    }
}
