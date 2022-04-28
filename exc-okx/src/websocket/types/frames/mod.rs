/// Client frame.
pub mod client;

/// Server frame.
pub mod server;

/// Frame.
pub trait Frame {
    /// Stream id.
    fn stream_id(&self) -> usize;
}
