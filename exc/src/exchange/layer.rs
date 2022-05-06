use std::marker::PhantomData;

use tower::Layer;

use crate::Exchange;

/// Exchange layer.
pub struct ExchangeLayer<Req> {
    _req: PhantomData<fn() -> Req>,
}

impl<Req> Default for ExchangeLayer<Req> {
    fn default() -> Self {
	Self {
	    _req: PhantomData,
	}
    }
}

impl<S, Req> Layer<S> for ExchangeLayer<Req>
{
    type Service = Exchange<S, Req>;

    fn layer(&self, inner: S) -> Self::Service {
	Exchange::new(inner)
    }
}
