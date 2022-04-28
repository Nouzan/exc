use super::super::messages::event::Event;

/// Server Frame.
#[derive(Debug, Clone)]
pub struct ServerFrame {
    pub(crate) stream_id: usize,
    /// Inner Event.
    pub inner: Event,
}
