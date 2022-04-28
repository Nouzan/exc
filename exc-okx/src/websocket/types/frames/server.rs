use super::super::messages::event::Event;
use super::Frame;

/// Server Frame.
#[derive(Debug, Clone)]
pub struct ServerFrame {
    pub(crate) stream_id: usize,
    pub(crate) inner: Event,
}

impl Frame for ServerFrame {
    fn stream_id(&self) -> usize {
        self.stream_id
    }
}
