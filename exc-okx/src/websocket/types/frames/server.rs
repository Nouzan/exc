use super::super::messages::event::{Event, ResponseKind};

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
