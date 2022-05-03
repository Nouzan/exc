use super::Exchange;
use crate::{
    error::ExchangeError,
    types::{subscriptions::SubscribeTickers, ticker::Ticker},
};
use futures::{
    future::{BoxFuture, FutureExt},
    stream::BoxStream,
    StreamExt, TryStreamExt,
};
use tower::{Service, ServiceExt};

impl<C, Req> Exchange<C, Req>
where
    Self: Service<
        SubscribeTickers,
        Response = BoxStream<'static, Result<Ticker, ExchangeError>>,
        Error = ExchangeError,
    >,
{
    /// Subscribe tickers.
    pub async fn subscribe_tickers(
        &mut self,
        inst: &str,
    ) -> Result<BoxStream<'static, Result<Ticker, ExchangeError>>, ExchangeError> {
        ServiceExt::<SubscribeTickers>::oneshot(self, SubscribeTickers::new(inst)).await
    }
}

impl<C, Req> Service<SubscribeTickers> for Exchange<C, Req>
where
    C: Service<Req, Error = ExchangeError>,
    Req: TryFrom<SubscribeTickers, Error = ExchangeError>,

    BoxStream<'static, Result<Ticker, ExchangeError>>: TryFrom<C::Response, Error = ExchangeError>,
    C::Future: Send + 'static,
{
    type Response = BoxStream<'static, Result<Ticker, ExchangeError>>;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.channel.poll_ready(cx).map_err(ExchangeError::from)
    }

    fn call(&mut self, req: SubscribeTickers) -> Self::Future {
        let request = Req::try_from(req);
        match request {
            Ok(req) => {
                let res = self.channel.call(req);
                async move {
                    let resp = res.await?;
                    let stream: BoxStream<'static, Result<Ticker, _>> = BoxStream::try_from(resp)?;
                    let stream = stream.map_err(ExchangeError::from).boxed();
                    Ok(stream)
                }
                .left_future()
            }
            Err(err) => futures::future::ready(Err(err)).right_future(),
        }
        .boxed()
    }
}
