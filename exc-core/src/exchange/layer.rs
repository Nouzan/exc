use std::marker::PhantomData;

use tower::Layer;

use crate::Adapt;

/// Adapt layer.
pub struct AdaptLayer<Req> {
    _req: PhantomData<fn() -> Req>,
}

impl<Req> Default for AdaptLayer<Req> {
    fn default() -> Self {
        Self { _req: PhantomData }
    }
}

impl<S, Req> Layer<S> for AdaptLayer<Req> {
    type Service = Adapt<S, Req>;

    fn layer(&self, inner: S) -> Self::Service {
        Adapt::new(inner)
    }
}
