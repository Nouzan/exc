use super::types::request::WsRequestMessage;
use super::types::Args;
use super::{types::envelope::Envelope, WsRequest, WsResponse};
use crate::error::OkxError;
use exc::transport::websocket::WsStream;
use futures::{future::BoxFuture, stream::SplitSink, FutureExt, SinkExt, StreamExt};
use std::collections::HashMap;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

type Callback = oneshot::Sender<Result<WsResponse, OkxError>>;

/// Okx websocket API service.
pub struct OkxWebsocketService {
    req_tx: mpsc::UnboundedSender<Envelope>,
}

const MESSAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

#[derive(Default)]
struct Dispatch {
    subscriptions: HashMap<Args, ()>,
    subscribing: HashMap<Args, Callback>,
    unsubscribing: HashMap<Args, Callback>,
}

impl Dispatch {
    async fn dispatch(
        &mut self,
        ws_tx: &mut SplitSink<WsStream, Message>,
        Envelope { request, callback }: Envelope,
    ) {
        match &request {
            WsRequest::Subscribe(args) => {
                if self.subscriptions.contains_key(args)
                    || self.subscribing.contains_key(args)
                    || self.unsubscribing.contains_key(args)
                {
                    let args = args.clone();
                    tokio::spawn(async move {
                        if let Err(err) =
                            callback.send(Err(OkxError::SubscribedOrUnsubscribing(args)))
                        {
                            error!("callback error: {err:?}");
                        }
                    });
                } else {
                    let args = args.clone();
                    let req = request.into();
                    match self.send_message(&req, ws_tx).await {
                        Ok(_) => {
                            self.subscribing.insert(args, callback);
                        }
                        Err(err) => {
                            tokio::spawn(async move {
                                if let Err(err) = callback.send(Err(err)) {
                                    error!("callback error: {err:?}");
                                }
                            });
                        }
                    }
                }
            }
            WsRequest::Unsubscribe(args) => {
                if self.subscribing.contains_key(args) || self.unsubscribing.contains_key(args) {
                    let args = args.clone();
                    tokio::spawn(async move {
                        if let Err(err) =
                            callback.send(Err(OkxError::SubscribingOrUnsubscribing(args)))
                        {
                            error!("callback error: {err:?}");
                        }
                    });
                } else {
                    let args = args.clone();
                    let req = request.into();
                    match self.send_message(&req, ws_tx).await {
                        Ok(_) => {
                            self.unsubscribing.insert(args, callback);
                        }
                        Err(err) => {
                            tokio::spawn(async move {
                                if let Err(err) = callback.send(Err(err)) {
                                    error!("callback error: {err:?}");
                                }
                            });
                        }
                    }
                }
            }
        }
    }

    async fn handle_message(
        &mut self,
        msg: Result<Message, tokio_tungstenite::tungstenite::Error>,
    ) -> Result<(), OkxError> {
        Ok(())
    }

    async fn send_message(
        &self,
        req: &WsRequestMessage,
        ws_tx: &mut SplitSink<WsStream, Message>,
    ) -> Result<(), OkxError> {
        let msg = req.to_websocket()?;
        ws_tx.send(msg).await?;
        Ok(())
    }
}

async fn conn_task(
    ws: WsStream,
    mut req_rx: mpsc::UnboundedReceiver<Envelope>,
) -> Result<(), OkxError> {
    let (mut tx, rx) = ws.split();
    let stream = tokio_stream::StreamExt::timeout(rx, MESSAGE_TIMEOUT);
    futures::pin_mut!(stream);
    let mut ping = false;
    let mut dispatch = Dispatch::default();
    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    Some(msg) => match msg {
                        Ok(msg) => {
                            ping = false;
                            debug!("received message: {msg:?}");
                            dispatch.handle_message(msg).await?;
                        },
                        Err(err) => {
                            debug!("message timeout: {err}");
                            if ping {
                                error!("ping timeout; exit");
                                return Err(OkxError::PingTimeout);
                            } else {
                                ping = true;
                                if let Err(err) = tx.send(Message::Text("ping".to_string())).await {
                                    return Err(OkxError::Ping(err));
                                }
                            }
                        }
                    },
                    None => {
                        return Err(OkxError::WebsocketDisconnected);
                    }
                }
            },
            req = req_rx.recv() => {
                match req {
                    Some(req) => {
                        dispatch.dispatch(&mut tx, req).await;
                    },
                    None => {
                        return Err(OkxError::RequestSenderDropped)
                    }
                }
            }
        }
    }
}

impl OkxWebsocketService {
    pub(crate) async fn init(ws: WsStream) -> Result<Self, OkxError> {
        let (req_tx, req_rx) = mpsc::unbounded_channel();
        let task = conn_task(ws, req_rx);
        tokio::spawn(async move {
            if let Err(err) = task.await {
                error!("connection error: {err}");
            }
        });
        Ok(Self { req_tx })
    }
}

impl tower::Service<WsRequest> for OkxWebsocketService {
    type Response = WsResponse;
    type Error = OkxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: WsRequest) -> Self::Future {
        let (req, resp) = Envelope::new(req);
        let res = self
            .req_tx
            .send(req)
            .map_err(|err| OkxError::Dispatch(err.0.request));
        async move {
            res?;
            resp.await?
        }
        .boxed()
    }
}
