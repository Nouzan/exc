use super::{callback::Callback, frames::client::ClientFrame, messages::Args};
use crate::{
    key::{Key, SignError},
    websocket::types::messages::request::WsRequest,
};
use async_stream::stream;
use exc_core::{
    types::{ticker::SubscribeTickers, PlaceOrder},
    ExchangeError,
};
use futures::stream::{BoxStream, StreamExt};
use std::collections::BTreeMap;

/// Okx websocket api request.
pub struct Request {
    cb: Callback,
    inner: BoxStream<'static, ClientFrame>,
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
        Self::subscribe(Args(BTreeMap::from([
            ("channel".to_string(), "tickers".to_string()),
            ("instId".to_string(), inst.to_string()),
        ])))
    }

    /// Subscribe to the given channel.
    pub fn subscribe(args: Args) -> Self {
        let (cb, rx) = Callback::new();
        let stream = stream! {
            yield ClientFrame { stream_id: 0, inner: WsRequest::Subscribe(args.clone()) };
            let _ = rx.await;
            yield ClientFrame { stream_id: 0, inner: WsRequest::Unsubscribe(args.clone()) };
        };

        Self {
            cb,
            inner: stream.boxed(),
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
