use exc_service::{ExcService, ExchangeError, Request};
use exc_types::{FetchInstruments, SubscribeInstruments};
use futures::{future::BoxFuture, FutureExt, StreamExt};
use tower::{Layer, Service, ServiceExt};

/// Fetch-Then-Subscribe instruments service.
#[derive(Debug, Clone, Copy)]
pub struct FetchThenSubscribeInstruments<S>(S);

impl<S> Service<SubscribeInstruments> for FetchThenSubscribeInstruments<S>
where
    S: Clone + Send + 'static,
    S: ExcService<FetchInstruments>,
    S: ExcService<SubscribeInstruments>,
    <S as ExcService<FetchInstruments>>::Future: Send,
    <S as ExcService<SubscribeInstruments>>::Future: Send,
{
    type Response = <SubscribeInstruments as Request>::Response;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::<FetchInstruments>::poll_ready(&mut self.0.as_service(), cx)
    }

    fn call(&mut self, req: SubscribeInstruments) -> Self::Future {
        let fetched = Service::<FetchInstruments>::call(
            &mut self.0.as_service(),
            FetchInstruments {
                tag: req.tag.clone(),
            },
        );
        let mut svc = self.0.clone();
        async move {
            let fetched = fetched.await?;
            let subscribed = Service::<SubscribeInstruments>::call(
                svc.as_service().ready().await?,
                SubscribeInstruments {
                    tag: req.tag.clone(),
                },
            )
            .await?;
            Ok(fetched.chain(subscribed).boxed())
        }
        .boxed()
    }
}

/// Fetch-Then-Subscribe instruments layer.
#[derive(Debug, Default)]
pub struct FetchThenSubscribeInstrumentsLayer;

impl<S> Layer<S> for FetchThenSubscribeInstrumentsLayer {
    type Service = FetchThenSubscribeInstruments<S>;
    fn layer(&self, inner: S) -> Self::Service {
        FetchThenSubscribeInstruments(inner)
    }
}
