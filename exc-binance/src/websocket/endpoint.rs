use std::{collections::HashSet, time::Duration};

use tower::{reconnect::Reconnect, ServiceExt};

use crate::types::Name;

use super::connect::{BinanceWsConnect, BinanceWsHost, BinanceWsTarget, Http};
use super::{error::WsError, protocol::WsClient, request::WsRequest, BinanceWebsocketApi};

const DEFAULT_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_STREAM_TIMEOUT: Duration = Duration::from_secs(30);

/// A builder of binance websocket api service.
#[derive(Clone)]
pub struct WsEndpoint {
    target: BinanceWsTarget,
    main_stream: HashSet<Name>,
    keep_alive_timeout: Option<Duration>,
    default_stream_timeout: Option<Duration>,
    listen_key_retry: Option<usize>,
    listen_key_refresh_interval: Option<Duration>,
}

impl WsEndpoint {
    /// Create a new binance websocket api endpoint.
    pub fn new(host: BinanceWsHost, name: Name) -> Self {
        Self {
            target: BinanceWsTarget {
                host,
                name: name.clone(),
                key_provider: None,
            },
            main_stream: HashSet::from([name]),
            keep_alive_timeout: None,
            default_stream_timeout: None,
            listen_key_retry: None,
            listen_key_refresh_interval: None,
        }
    }

    /// Set the keep-alive timeout.
    pub fn keep_alive_timeout(&mut self, duration: Duration) -> &mut Self {
        self.keep_alive_timeout = Some(duration);
        self
    }

    /// Set the default stream timeout for each request stream.
    pub fn default_stream_timeout(&mut self, duration: Duration) -> &mut Self {
        self.default_stream_timeout = Some(duration);
        self
    }

    /// Set listen key retrys.
    pub fn listen_key_retry(&mut self, retry: usize) -> &mut Self {
        self.listen_key_retry = Some(retry);
        self
    }

    /// Set listen key refresh interval.
    pub fn listen_key_refresh_interval(&mut self, interval: Duration) -> &mut Self {
        self.listen_key_refresh_interval = Some(interval);
        self
    }

    /// Private endpoint of USD-M Futures API.
    pub(crate) fn private(&mut self, http: Http) -> &mut Self {
        self.target.host.private();
        self.target.key_provider = Some(http);
        // self.add_main_stream(Name::order_trade_update());
        self
    }

    /// Add main stream.
    pub(crate) fn _add_main_stream(&mut self, name: Name) -> &mut Self {
        self.main_stream.insert(name);
        self
    }

    /// Connect to binance websocket api.
    pub fn connect(&self) -> BinanceWebsocketApi {
        let main_stream = self.main_stream.clone();
        let keep_alive_timeout = self
            .keep_alive_timeout
            .unwrap_or(DEFAULT_KEEP_ALIVE_TIMEOUT);
        let default_stream_timeout = self
            .default_stream_timeout
            .unwrap_or(DEFAULT_STREAM_TIMEOUT);
        let connect = BinanceWsConnect {
            main_stream,
            keep_alive_timeout,
            default_stream_timeout,
            retry: self.listen_key_retry,
            interval: self.listen_key_refresh_interval,
        };
        let connection = Reconnect::new::<WsClient, WsRequest>(connect, self.target.clone())
            .map_err(|err| match err.downcast::<WsError>() {
                Ok(err) => *err,
                Err(err) => WsError::UnknownConnection(err),
            });
        BinanceWebsocketApi {
            svc: connection.boxed(),
        }
    }
}
