use super::{request::WsRequest, response::WsResponse};
use crate::error::OkxError;
use exc::transport::websocket::WsStream;
use futures::{future::BoxFuture, SinkExt, StreamExt};
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

type Request = ();

/// Okx websocket API service.
pub struct OkxWebsocketService {
    dispatch: mpsc::UnboundedSender<Request>,
}

const MESSAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

async fn conn_task(
    ws: WsStream,
    mut requests: mpsc::UnboundedReceiver<Request>,
) -> Result<(), OkxError> {
    let (mut tx, rx) = ws.split();
    let stream = tokio_stream::StreamExt::timeout(rx, MESSAGE_TIMEOUT);
    futures::pin_mut!(stream);
    let mut ping = false;
    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(msg) => match msg {
                        Ok(msg) => {
                            ping = false;
                            debug!("received message: {msg:?}");
                        },
                        Err(err) => {
                            debug!("message timeout: {err}");
                            if ping {
                                error!("ping timeout; exit");
                                return Err(OkxError::PingTimeout);
                            } else {
                                ping = true;
                                tx.send(Message::Text("ping".to_string())).await?;
                            }
                        }
                    },
                    None => {
                        return Err(OkxError::WebsocketDisconnected);
                    }
                }
            },
            _req = requests.recv() => {

            }
        }
    }
}

impl OkxWebsocketService {
    pub(crate) async fn init(ws: WsStream) -> Result<Self, OkxError> {
        let (dispatch, requests) = mpsc::unbounded_channel();
        let task = conn_task(ws, requests);
        tokio::spawn(async move {
            if let Err(err) = task.await {
                error!("connection error: {err}");
            }
        });
        Ok(Self { dispatch })
    }
}

impl tower::Service<WsRequest> for OkxWebsocketService {
    type Response = WsResponse;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: WsRequest) -> Self::Future {
        todo!()
    }
}
