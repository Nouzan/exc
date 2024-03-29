use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{Future, Sink, Stream};
use tokio::time::{Instant, Sleep};
use tokio_tungstenite::tungstenite::Message;

use crate::websocket::error::WsError;

/// Keep-alive protocol layer.
pub fn layer<T>(
    transport: T,
    timeout: Duration,
) -> impl Sink<String, Error = WsError> + Stream<Item = Result<String, WsError>>
where
    T: Sink<Message, Error = WsError>,
    T: Stream<Item = Result<Message, WsError>>,
{
    let deadline = tokio::time::sleep_until(Instant::now() + timeout);
    KeepAlive {
        inner: transport,
        deadline,
        duration: timeout,
        close: false,
        state: PingState::Idle,
    }
}

#[derive(Debug, Clone)]
enum PingState {
    Idle,
    WaitReady(Vec<u8>),
    WaitSent,
}

// Fork from [`tokio_stream::Timeout`]
pin_project_lite::pin_project! {
    pub(super) struct KeepAlive<T> {
        #[pin]
        inner: T,
        #[pin]
        deadline: Sleep,
        duration: Duration,
        close: bool,
        state: PingState,
    }
}

impl<T> Sink<String> for KeepAlive<T>
where
    T: Sink<Message>,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        match futures::ready!(this.inner.as_mut().poll_ready(cx)) {
            Ok(()) => {
                if let PingState::WaitReady(msg) = this.state {
                    let msg = Message::Pong(std::mem::take(msg));
                    *this.state = PingState::WaitSent;
                    tracing::trace!("keep-alive; {:?}", this.state);
                    if let Err(err) = this.inner.start_send(msg) {
                        *this.close = true;
                        return Poll::Ready(Err(err));
                    }
                }
                Poll::Ready(Ok(()))
            }
            Err(err) => {
                *this.close = true;
                Poll::Ready(Err(err))
            }
        }
    }

    fn start_send(self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        self.project().inner.start_send(Message::Text(item))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        match futures::ready!(this.inner.poll_flush(cx)) {
            Ok(()) => {
                if let PingState::WaitSent = this.state {
                    *this.state = PingState::Idle;
                    tracing::trace!("keep-alive; {:?}", this.state);
                }
                Poll::Ready(Ok(()))
            }
            Err(err) => {
                *this.close = true;
                Poll::Ready(Err(err))
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<T> Stream for KeepAlive<T>
where
    T: Sink<Message, Error = WsError>,
    T: Stream<Item = Result<Message, WsError>>,
{
    type Item = Result<String, WsError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        if *this.close {
            return Poll::Ready(None);
        }
        tracing::trace!("keep-alive; poll next message, state={:?}", this.state);
        loop {
            match this.state {
                PingState::WaitReady(msg) => match this.inner.as_mut().poll_ready(cx) {
                    Poll::Ready(res) => {
                        if let Err(err) = res {
                            *this.close = true;
                            return Poll::Ready(Some(Err(err)));
                        }
                        let msg = Message::Pong(std::mem::take(msg));
                        *this.state = PingState::WaitSent;
                        tracing::trace!("keep-alive; {:?}", this.state);
                        if let Err(err) = this.inner.as_mut().start_send(msg) {
                            *this.close = true;
                            return Poll::Ready(Some(Err(err)));
                        }
                    }
                    Poll::Pending => {
                        break;
                    }
                },
                PingState::WaitSent => match this.inner.as_mut().poll_flush(cx) {
                    Poll::Ready(res) => {
                        if let Err(err) = res {
                            *this.close = true;
                            return Poll::Ready(Some(Err(err)));
                        }
                        *this.state = PingState::Idle;
                        tracing::trace!("keep-alive; {:?}", this.state);
                        break;
                    }
                    Poll::Pending => {
                        break;
                    }
                },
                _ => {
                    break;
                }
            }
        }

        loop {
            match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(msg) => match msg {
                    Some(msg) => {
                        let next = Instant::now() + *this.duration;
                        this.deadline.as_mut().reset(next);
                        tracing::trace!("keep-alive; deadline reset");
                        match msg {
                            Ok(Message::Ping(msg)) => {
                                tracing::trace!("keep-alive; received ping: {msg:?}");
                                loop {
                                    match this.state {
                                        PingState::Idle => {
                                            *this.state = PingState::WaitReady(msg.clone());
                                            tracing::trace!("keep-alive; {:?}", this.state);
                                        }
                                        PingState::WaitReady(msg) => {
                                            if let Poll::Ready(res) =
                                                this.inner.as_mut().poll_ready(cx)
                                            {
                                                if let Err(err) = res {
                                                    *this.close = true;
                                                    return Poll::Ready(Some(Err(err)));
                                                }
                                                let msg = Message::Pong(std::mem::take(msg));
                                                *this.state = PingState::WaitSent;
                                                tracing::trace!("keep-alive; {:?}", this.state);
                                                if let Err(err) =
                                                    this.inner.as_mut().start_send(msg)
                                                {
                                                    return Poll::Ready(Some(Err(err)));
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                        PingState::WaitSent => {
                                            if let Poll::Ready(res) =
                                                this.inner.as_mut().poll_flush(cx)
                                            {
                                                if let Err(err) = res {
                                                    *this.close = true;
                                                    return Poll::Ready(Some(Err(err)));
                                                }
                                                *this.state = PingState::Idle;
                                                tracing::trace!("keep-alive; {:?}", this.state);
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                            Ok(Message::Text(msg)) => {
                                tracing::trace!("keep-alive; new text frame: {msg}");
                                return Poll::Ready(Some(Ok(msg)));
                            }
                            Ok(Message::Close(msg)) => {
                                tracing::error!("keep-alive; remote close: {msg:?}");
                                return Poll::Ready(Some(Err(WsError::RemoteClose)));
                            }
                            Err(err) => {
                                return Poll::Ready(Some(Err(err)));
                            }
                            Ok(msg) => {
                                tracing::trace!("keep-alive; received other message: {msg:?}");
                            }
                        }
                    }
                    None => {
                        return Poll::Ready(None);
                    }
                },
                Poll::Pending => {
                    tracing::trace!("keep-alive; pending message");
                    break;
                }
            }
        }

        futures::ready!(this.deadline.poll(cx));
        *this.close = true;
        Poll::Ready(Some(Err(WsError::TransportTimeout)))
    }
}
