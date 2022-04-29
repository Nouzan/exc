use crate::websocket::types::messages::{
    event::Event,
    request::{WsRequest, WsRequestMessage},
};
use futures::{Sink, SinkExt, Stream, StreamExt};
use thiserror::Error;

/// Okx websocket message layer errors.
#[derive(Debug, Error)]
pub enum MessageError<E> {
    /// Json error.
    #[error("[message] serializing: {0}")]
    Serializing(serde_json::Error),
    /// Transport error.
    #[error("{0}")]
    Transport(#[from] E),
}

pub(super) fn layer<T, E>(
    transport: T,
) -> impl Sink<WsRequest, Error = MessageError<E>> + Stream<Item = Result<Event, MessageError<E>>>
where
    T: Sink<String, Error = E>,
    T: Stream<Item = Result<String, E>>,
{
    transport
        .sink_map_err(MessageError::from)
        .with(|msg: WsRequest| async move {
            let msg: WsRequestMessage = msg.into();
            let msg = serde_json::to_string(&msg).map_err(MessageError::Serializing)?;
            Ok(msg)
        })
        .filter_map(|msg| async move {
            match msg {
                Ok(msg) => match serde_json::from_str::<Event>(&msg) {
                    Ok(event) => {
                        trace!("message layer; received event={event:?}");
                        Some(Ok(event))
                    }
                    Err(err) => {
                        warn!("message layer; deserializing message error: {err}; ignored");
                        None
                    }
                },
                Err(err) => Some(Err(err.into())),
            }
        })
}
