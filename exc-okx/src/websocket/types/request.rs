use super::{callback::Callback, frames::client::ClientFrame, messages::Args};
use crate::{
    key::{OkxKey as Key, SignError},
    websocket::types::messages::request::WsRequest,
};
use async_stream::stream;
use exc_core::{
    types::{
        ticker::{SubscribeStatistics, SubscribeTickers},
        PlaceOrder,
    },
    ExchangeError,
};
use futures::stream::{empty, BoxStream, StreamExt};

/// Okx websocket api request.
pub struct Request {
    cb: Callback,
    inner: BoxStream<'static, ClientFrame>,
    pub(crate) reconnect: bool,
}

impl Request {
    pub(crate) fn into_client_stream(self) -> ClientStream {
        ClientStream {
            id: 0,
            cb: Some(self.cb),
            inner: self.inner,
        }
    }

    /// Subscribe tickers.
    pub fn subscribe_tickers(inst: &str) -> Self {
        Self::subscribe(Args::subscribe_tickers(inst))
    }

    /// Subscribe orders.
    pub fn subscribe_orders(inst: &str) -> Self {
        Self::subscribe(Args::subscribe_orders(inst))
    }

    /// Subscribe to the given channel.
    pub fn subscribe(args: Args) -> Self {
        let (cb, rx) = Callback::new();
        let stream = stream! {
            yield ClientFrame { stream_id: 0, inner: WsRequest::Subscribe(args.clone()) };
            let _ = rx.await;
            yield ClientFrame { stream_id: 0, inner: WsRequest::Unsubscribe(args) };
        };

        Self {
            cb,
            inner: stream.boxed(),
            reconnect: false,
        }
    }

    /// Login request.
    pub(crate) fn login(key: Key) -> Result<Self, SignError> {
        let (cb, _rx) = Callback::new();
        let signature = key.sign_now("GET", "/users/self/verify", true)?;
        let stream = stream! {
            yield ClientFrame { stream_id: 0, inner: WsRequest::login(key, signature) };
            // let _ = rx.await;
        };

        Ok(Self {
            cb,
            inner: stream.boxed(),
            reconnect: false,
        })
    }

    /// Order request.
    pub fn order(req: &PlaceOrder) -> Self {
        let (cb, _rx) = Callback::new();
        let opts = req.opts.clone();
        let place = req.place;
        let stream = stream! {
            yield ClientFrame { stream_id: 0, inner: WsRequest::order(&place, &opts) };
            // let _ = rx.await;
        };

        Self {
            cb,
            inner: stream.boxed(),
            reconnect: false,
        }
    }

    /// Cancel order request.
    pub fn cancel_order(inst: &str, id: &str) -> Self {
        let (cb, _rx) = Callback::new();
        let inst = inst.to_string();
        let id = id.to_string();
        let stream = stream! {
            yield ClientFrame { stream_id: 0, inner: WsRequest::cancel_order(&inst, &id) };
            // let _ = rx.await;
        };

        Self {
            cb,
            inner: stream.boxed(),
            reconnect: false,
        }
    }

    /// Reconnect.
    pub fn reconnect() -> Self {
        let (cb, _rx) = Callback::new();
        Self {
            cb,
            inner: empty().boxed(),
            reconnect: true,
        }
    }
}

/// Client stream.
pub struct ClientStream {
    pub(crate) id: usize,
    pub(crate) cb: Option<Callback>,
    pub(crate) inner: BoxStream<'static, ClientFrame>,
}

impl TryFrom<SubscribeTickers> for Request {
    type Error = ExchangeError;

    fn try_from(value: SubscribeTickers) -> Result<Self, Self::Error> {
        let inst = value.instrument;
        Ok(Self::subscribe_tickers(&inst))
    }
}

impl TryFrom<SubscribeStatistics> for Request {
    type Error = ExchangeError;

    fn try_from(value: SubscribeStatistics) -> Result<Self, Self::Error> {
        let inst = value.instrument;
        Ok(Self::subscribe_tickers(&inst))
    }
}
