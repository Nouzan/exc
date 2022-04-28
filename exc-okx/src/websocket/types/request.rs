use std::collections::BTreeMap;

use crate::websocket::WsRequest;

use super::{frames::client::ClientFrame, messages::Args};
use async_stream::stream;
use futures::stream::{BoxStream, StreamExt};
use tokio::sync::oneshot;

/// Subscription.
pub struct Subscription {
    tx: Option<oneshot::Sender<()>>,
}

impl Subscription {
    /// Unsubscribe the channel.
    pub fn unsubscribe(mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Okx websocket api request.
pub struct Request {
    inner: BoxStream<'static, ClientFrame>,
}

impl Request {
    pub(crate) fn into_client_stream(self) -> ClientStream {
        ClientStream {
            id: 0,
            inner: self.inner,
        }
    }

    /// Subscribe tickers.
    pub fn subscribe_tickers(inst: &str) -> (Self, Subscription) {
        Self::subscribe(Args(BTreeMap::from([
            ("channel".to_string(), "tickers".to_string()),
            ("instId".to_string(), inst.to_string()),
        ])))
    }

    /// Subscribe to the given channel.
    pub fn subscribe(args: Args) -> (Self, Subscription) {
        let (tx, rx) = oneshot::channel::<()>();
        let stream = stream! {
            yield ClientFrame { stream_id: 0, inner: WsRequest::Subscribe(args.clone()) };
            let _ = rx.await;
            yield ClientFrame { stream_id: 0, inner: WsRequest::Unsubscribe(args.clone()) };
        };
        (
            Self {
                inner: stream.boxed(),
            },
            Subscription { tx: Some(tx) },
        )
    }
}

// impl Drop for Subscription {
//     fn drop(&mut self) {
//         if let Some(tx) = self.tx.take() {
//             let _ = tx.send(());
//         }
//     }
// }

/// Client stream.
pub struct ClientStream {
    pub(crate) id: usize,
    pub(crate) inner: BoxStream<'static, ClientFrame>,
}
