use super::types::request::WsRequestMessage;
use super::types::{event::Event, Args};
use super::{types::envelope::Envelope, WsRequest, WsResponse};
use crate::error::OkxError;
use crate::websocket::types::event::ResponseKind;
use exc::transport::websocket::WsStream;
use futures::{future::BoxFuture, stream::SplitSink, FutureExt, SinkExt, StreamExt};
use std::collections::{HashMap, VecDeque};
use std::task::{Context, Poll};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

type Callback = oneshot::Sender<Result<WsResponse, OkxError>>;

/// Okx websocket API service.
pub struct OkxWebsocketService {
    req_tx: mpsc::UnboundedSender<Envelope>,
}

const MESSAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

#[derive(Debug)]
enum InFlight {
    Subscribe(Args),
    Unsubscribe(Args),
}

#[derive(Default)]
struct Dispatch {
    subscriptions: HashMap<Args, ()>,
    subscribing: HashMap<Args, Callback>,
    unsubscribing: HashMap<Args, Callback>,
    in_flights: VecDeque<InFlight>,
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
                            self.subscribing.insert(args.clone(), callback);
                            self.in_flights.push_back(InFlight::Subscribe(args));
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
                            self.unsubscribing.insert(args.clone(), callback);
                            self.in_flights.push_back(InFlight::Unsubscribe(args));
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

    async fn handle_message(&mut self, msg: Result<Message, WsError>) -> Result<(), OkxError> {
        match msg {
            Ok(msg) => match msg {
                Message::Text(msg) => match msg.as_str() {
                    "pong" => {}
                    msg => match serde_json::from_str::<Event>(msg) {
                        Ok(event) => {
                            debug!("received an event: {event:?}");
                            match event {
                                Event::Response(response) => {
                                    let in_flight =
                                        self.in_flights.pop_front().ok_or_else(|| {
                                            OkxError::BrokenChannle(anyhow::anyhow!(
                                                "unwanted response"
                                            ))
                                        })?;
                                    match response {
                                        ResponseKind::Login(_) => {
                                            error!("unimplemented");
                                        }
                                        ResponseKind::Subscribe { arg } => {
                                            if !matches!(in_flight, InFlight::Subscribe(_)) {
                                                return Err(OkxError::BrokenChannle(anyhow::anyhow!("response mismatched: in_flight={in_flight:?}")));
                                            }
                                            if let Some(callback) = self.subscribing.remove(&arg) {
                                                self.subscriptions.insert(arg, ());
                                                tokio::spawn(async move {
                                                    if let Err(err) =
                                                        callback.send(Ok(WsResponse {}))
                                                    {
                                                        error!("callback error: {err:?}");
                                                    }
                                                });
                                            }
                                        }
                                        ResponseKind::Unsubscribe { arg } => {
                                            if !matches!(in_flight, InFlight::Unsubscribe(_)) {
                                                return Err(OkxError::BrokenChannle(anyhow::anyhow!("response mismatched: in_flight={in_flight:?}")));
                                            }
                                            if let Some(callback) = self.unsubscribing.remove(&arg)
                                            {
                                                self.subscriptions.remove(&arg);
                                                if let Err(err) = callback.send(Ok(WsResponse {})) {
                                                    error!("callback error: {err:?}");
                                                }
                                            }
                                        }
                                        ResponseKind::Error(msg) => match &in_flight {
                                            InFlight::Subscribe(args) => {
                                                if let Some(callback) =
                                                    self.subscribing.remove(args)
                                                {
                                                    tokio::spawn(async move {
                                                        if let Err(err) = callback.send(Err(
                                                            OkxError::Api(msg.to_string()),
                                                        )) {
                                                            error!("callback error: {err:?}");
                                                        }
                                                    });
                                                } else {
                                                    return Err(OkxError::BrokenChannle(anyhow::anyhow!("response mismatched: in_flight={in_flight:?}")));
                                                }
                                            }
                                            InFlight::Unsubscribe(args) => {
                                                if let Some(callback) =
                                                    self.unsubscribing.remove(args)
                                                {
                                                    tokio::spawn(async move {
                                                        if let Err(err) = callback.send(Err(
                                                            OkxError::Api(msg.to_string()),
                                                        )) {
                                                            error!("callback error: {err:?}");
                                                        }
                                                    });
                                                } else {
                                                    return Err(OkxError::BrokenChannle(anyhow::anyhow!("response mismatched: in_flight={in_flight:?}")));
                                                }
                                            }
                                        },
                                    }
                                }
                                Event::Change(change) => {
                                    info!("change: {change:?}");
                                }
                            }
                        }
                        Err(err) => {
                            error!("deserializing error: msg={msg} err={err}");
                            return Err(err.into());
                        }
                    },
                },
                Message::Close(_) => {
                    return Err(OkxError::WebsocketClosed);
                }
                _ => {}
            },
            Err(err) => match &err {
                WsError::ConnectionClosed | WsError::AlreadyClosed => {
                    return Err(err.into());
                }
                _ => {
                    error!("websocket error: {err}");
                }
            },
        }
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
