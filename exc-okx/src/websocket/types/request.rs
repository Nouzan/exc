use std::collections::BTreeMap;
use crate::websocket::types::messages::request::WsRequest;
use super::{callback::Callback, frames::client::ClientFrame, messages::Args};
use async_stream::stream;
use futures::stream::{BoxStream, StreamExt};

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
}

/// Client stream.
pub struct ClientStream {
    pub(crate) id: usize,
    pub(crate) cb: Option<Callback>,
    pub(crate) inner: BoxStream<'static, ClientFrame>,
}
