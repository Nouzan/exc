use std::marker::PhantomData;

use tower::Layer;

use crate::{ExcService, Request};

use super::Exc;

/// Build [`Exc`] from an [`ExcService`].
pub struct ExcLayer<Req> {
    _req: PhantomData<fn() -> Req>,
}

impl<Req> Default for ExcLayer<Req> {
    fn default() -> Self {
        Self { _req: PhantomData }
    }
}

impl<S, Req> Layer<S> for ExcLayer<Req>
where
    Req: Request,
    S: ExcService<Req>,
{
    type Service = Exc<S, Req>;

    fn layer(&self, inner: S) -> Self::Service {
        Exc::new(inner)
    }
}
