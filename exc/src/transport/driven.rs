use futures::{Sink, Stream, StreamExt};
use pin_project_lite::pin_project;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pin_project! {
    /// Helper for driven the inner transport.
    pub struct Driven<Req, E, Resp> {
        #[pin]
        sink: Pin<Box<dyn Sink<Req, Error=E> + Send>>,
        #[pin]
        stream: UnboundedReceiverStream<Resp>,
    }
}

impl<Req, E, Resp> Driven<Req, E, Resp> {
    /// Drive the given transport.
    pub fn new<T>(transport: T) -> Driven<Req, E, Resp>
    where
        Req: 'static + Send,
        Resp: 'static + Send,
        T: 'static + Sink<Req, Error = E> + Stream<Item = Resp> + Send,
    {
        let (stream_tx, stream_rx) = mpsc::unbounded_channel();
        let (sink, mut stream) = transport.split();
        let worker = async move {
            while let Some(resp) = stream.next().await {
                if stream_tx.send(resp).is_err() {
                    tracing::error!("driven sender is broken");
                    break;
                }
            }
            tracing::trace!("driven worker; stream is dead");
        };
        tokio::spawn(async move { worker.await });
        Driven {
            sink: Box::pin(sink),
            stream: UnboundedReceiverStream::new(stream_rx),
        }
    }
}

impl<Req, E, Resp> Sink<Req> for Driven<Req, E, Resp> {
    type Error = E;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().sink.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Req) -> Result<(), Self::Error> {
        self.project().sink.start_send(item)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().sink.poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().sink.poll_flush(cx)
    }
}

impl<Req, E, Resp> Stream for Driven<Req, E, Resp> {
    type Item = Resp;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
