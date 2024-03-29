use crate::websocket::types::{
    frames::{client::ClientFrame, server::ServerFrame},
    messages::{
        event::{Event, ResponseKind, TradeResponse},
        request::WsRequest,
    },
};
use futures::{ready, Sink, SinkExt, Stream, TryStreamExt};
use pin_project_lite::pin_project;
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

const LOGIN_TAG: &str = "login:login";

/// Frame layer errors.
#[derive(Debug, Error)]
pub enum FrameError<E> {
    /// Transport errors.
    #[error(transparent)]
    Transport(#[from] E),
}

fn client_message_to_tag(msg: &WsRequest) -> String {
    match msg {
        WsRequest::Subscribe(args) | WsRequest::Unsubscribe(args) => args.to_tag(),
        WsRequest::Login(_) => LOGIN_TAG.to_string(),
        WsRequest::Order(id, _) | WsRequest::CancelOrder(id, _) => id.clone(),
    }
}

fn server_message_to_tag(msg: &Event) -> Option<String> {
    match msg {
        Event::Change(change) => Some(change.arg.to_tag()),
        Event::Response(resp) => match resp {
            ResponseKind::Subscribe { arg } | ResponseKind::Unsubscribe { arg } => {
                Some(arg.to_tag())
            }
            ResponseKind::Login(_) => Some(LOGIN_TAG.to_string()),
            ResponseKind::Error(err) => match err.code.as_str() {
                "60001" | "60002" | "60003" | "60004" | "60005" | "60006" | "60007" | "60008"
                | "60009" | "60024" => Some(LOGIN_TAG.to_string()),
                _ => {
                    tracing::error!("failed to extract tag from api error: {err}");
                    None
                }
            },
        },
        Event::TradeResponse(resp) => match resp {
            TradeResponse::Order { id, .. } | TradeResponse::CancelOrder { id, .. } => {
                Some(id.clone())
            }
        },
    }
}

pub(super) fn layer<T, E>(
    transport: T,
) -> impl Sink<ClientFrame, Error = FrameError<E>> + Stream<Item = Result<ServerFrame, FrameError<E>>>
where
    T: Sink<WsRequest, Error = E>,
    T: Stream<Item = Result<Event, E>>,
{
    let inner = transport
        .sink_map_err(FrameError::from)
        .map_err(FrameError::from);
    Frame {
        inner,
        translate: HashMap::default(),
    }
}

pin_project! {
    struct Frame<T> {
        translate: HashMap<String, usize>,
        #[pin]
        inner: T,
    }
}

impl<T, E> Sink<ClientFrame> for Frame<T>
where
    T: Sink<WsRequest, Error = FrameError<E>>,
{
    type Error = FrameError<E>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: ClientFrame) -> Result<(), Self::Error> {
        let this = self.project();
        let msg = item.inner;
        let id = item.stream_id;
        let tag = client_message_to_tag(&msg);
        tracing::trace!(stream_id=%id, tag=%tag, "tagging client frame");
        this.translate.insert(tag, id);
        this.inner.start_send(msg)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<T, E> Stream for Frame<T>
where
    T: Stream<Item = Result<Event, FrameError<E>>>,
{
    type Item = Result<ServerFrame, FrameError<E>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            match ready!(this.inner.as_mut().poll_next(cx)) {
                Some(msg) => match msg {
                    Ok(msg) => {
                        if let Some(tag) = server_message_to_tag(&msg) {
                            tracing::trace!(tag=%tag, "matching server frame");
                            if let Some(id) = this.translate.get(&tag) {
                                tracing::trace!(stream_id=%id, tag=%tag, "matched server frame");
                                return Poll::Ready(Some(Ok(ServerFrame {
                                    stream_id: *id,
                                    inner: msg,
                                })));
                            }
                        }
                    }
                    Err(err) => {
                        return Poll::Ready(Some(Err(err)));
                    }
                },
                None => {
                    return Poll::Ready(None);
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
