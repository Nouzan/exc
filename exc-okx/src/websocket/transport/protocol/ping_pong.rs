use futures::{ready, Future, Sink, Stream};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;
use tokio::time::{Duration, Instant, Sleep};

/// Ping-Pong Errors.
#[derive(Debug, Error)]
pub enum PingPongError {
    /// Transport.
    #[error("[ping] transport: {0}")]
    Transport(#[from] anyhow::Error),
    /// Remote close.
    #[error("[ping] transport: remote closed")]
    RemoteClosed,
    /// Ping.
    #[error("[ping] ping: {0}")]
    Ping(anyhow::Error),
    /// Ping timeout.
    #[error("[ping] ping timeout")]
    PingTimeout,
    /// Ping already failed.
    #[error("[ping] ping already failed")]
    PingAlreadyFailed,
}

pub(super) fn layer<T, E>(
    transport: T,
) -> impl Sink<String, Error = PingPongError> + Stream<Item = Result<String, PingPongError>>
where
    T: Stream<Item = Result<String, E>>,
    T: Sink<String, Error = E>,
    E: Into<anyhow::Error>,
{
    PingPong::new(transport)
}

#[derive(Clone, Copy)]
enum PingState {
    Idle,
    Ping,
    PingSent,
    WaitPong,
    PingFailed,
}

pin_project! {
    pub(super) struct PingPong<S> {
        #[pin]
        inner: S,
        #[pin]
        message_deadline: Sleep,
        #[pin]
        ping_deadline: Sleep,
        state: PingState,
        close: bool,
    }
}

impl<S> PingPong<S> {
    const MESSAGE_TIMEOUT: Duration = Duration::from_secs(20);
    const PING: &'static str = "ping";

    fn new(inner: S) -> Self {
        let next = Instant::now() + Self::MESSAGE_TIMEOUT;
        let message_deadline = tokio::time::sleep_until(next);
        let ping_deadline = tokio::time::sleep_until(next);
        Self {
            inner,
            message_deadline,
            ping_deadline,
            state: PingState::Idle,
            close: false,
        }
    }
}

impl<T, E, S> Sink<T> for PingPong<S>
where
    S: Sink<T, Error = E>,
    E: Into<anyhow::Error>,
{
    type Error = PingPongError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .inner
            .poll_ready(cx)
            .map_err(|err| PingPongError::Transport(err.into()))
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.project()
            .inner
            .start_send(item)
            .map_err(|err| PingPongError::Transport(err.into()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .inner
            .poll_flush(cx)
            .map_err(|err| PingPongError::Transport(err.into()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .inner
            .poll_close(cx)
            .map_err(|err| PingPongError::Transport(err.into()))
    }
}

impl<S, Err> Stream for PingPong<S>
where
    S: Stream<Item = Result<String, Err>>,
    S: Sink<String, Error = Err>,
    Err: Into<anyhow::Error>,
{
    type Item = Result<String, PingPongError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        if *this.close {
            return Poll::Ready(None);
        }
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(s) => match s {
                Some(Ok(s)) => {
                    let next = Instant::now() + Self::MESSAGE_TIMEOUT;
                    this.message_deadline.reset(next);
                    *this.state = PingState::Idle;
                    trace!("ping pong; timer reset");
                    match s.as_str() {
                        "pong" => return Poll::Pending,
                        _ => return Poll::Ready(Some(Ok(s))),
                    }
                }
                Some(Err(err)) => {
                    return Poll::Ready(Some(Err(PingPongError::Transport(err.into()))));
                }
                None => {
                    *this.close = true;
                    trace!("ping pong; stream is dead");
                    return Poll::Ready(Some(Err(PingPongError::RemoteClosed)));
                }
            },
            Poll::Pending => {}
        };
        loop {
            match this.state {
                PingState::Idle => {
                    ready!(this.message_deadline.as_mut().poll(cx));
                    trace!("ping pong; need ping");
                    let next = Instant::now() + Self::MESSAGE_TIMEOUT;
                    this.ping_deadline.as_mut().reset(next);
                    *this.state = PingState::Ping;
                }
                PingState::Ping => match this.inner.as_mut().poll_ready(cx) {
                    Poll::Ready(_) => {
                        if let Err(err) = this.inner.as_mut().start_send(Self::PING.to_string()) {
                            let err = PingPongError::Transport(err.into());
                            trace!("ping pong; ping sent failed");
                            *this.state = PingState::PingFailed;
                            *this.close = true;
                            return Poll::Ready(Some(Err(err)));
                        }
                        *this.state = PingState::PingSent;
                        trace!("ping pong; ready to send ping");
                    }
                    Poll::Pending => {
                        ready!(this.ping_deadline.as_mut().poll(cx));
                        trace!("ping pong; ping timeout");
                        *this.state = PingState::PingFailed;
                        *this.close = true;
                        return Poll::Ready(Some(Err(PingPongError::PingTimeout)));
                    }
                },
                PingState::PingSent => match this.inner.as_mut().poll_flush(cx) {
                    Poll::Ready(_) => {
                        trace!("ping pong; ping sent");
                        *this.state = PingState::WaitPong;
                    }
                    Poll::Pending => {
                        ready!(this.ping_deadline.as_mut().poll(cx));
                        trace!("ping pong; ping timeout");
                        *this.state = PingState::PingFailed;
                        *this.close = true;
                        return Poll::Ready(Some(Err(PingPongError::PingTimeout)));
                    }
                },
                PingState::WaitPong => {
                    ready!(this.ping_deadline.as_mut().poll(cx));
                    trace!("ping pong; ping timeout");
                    *this.state = PingState::PingFailed;
                    *this.close = true;
                    return Poll::Ready(Some(Err(PingPongError::PingTimeout)));
                }
                PingState::PingFailed => {
                    trace!("ping pong; ping failed");
                    *this.close = true;
                    return Poll::Ready(Some(Err(PingPongError::PingAlreadyFailed)));
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
