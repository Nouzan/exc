use crate::error::OkxError;

use super::super::messages::event::{Event, ResponseKind};
use exc::types::ticker::Ticker;

/// Server Frame.
#[derive(Debug, Clone)]
pub struct ServerFrame {
    pub(crate) stream_id: usize,
    /// Inner Event.
    pub inner: Event,
}

impl ServerFrame {
    pub(crate) fn is_end_stream(&self) -> bool {
        matches!(
            self.inner,
            Event::Response(ResponseKind::Unsubscribe { arg: _ } | ResponseKind::Error(_))
        )
    }
}

impl TryFrom<ServerFrame> for Vec<Result<Ticker, OkxError>> {
    type Error = OkxError;

    fn try_from(value: ServerFrame) -> Result<Self, Self::Error> {
        value.inner.try_into()
    }
}
