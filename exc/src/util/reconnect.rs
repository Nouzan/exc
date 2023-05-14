use exc_core::{types::utils::Reconnect, ExcService, ExchangeError};
use futures::{
    future::{ready, BoxFuture},
    stream::iter,
    FutureExt, StreamExt,
};
use tower::ServiceExt;

enum State {
    Init,
    Reconnect,
}

/// Force reconnect service.
pub trait ReconnectService {
    /// Force reconnect.
    fn reconnect(&mut self) -> BoxFuture<'_, Result<(), ExchangeError>>;
}

impl<S> ReconnectService for S
where
    S: ExcService<Reconnect> + Send,
    S::Future: Send,
{
    fn reconnect(&mut self) -> BoxFuture<'_, Result<(), ExchangeError>>
    where
        Self: Sized,
    {
        let mut state = State::Init;
        ServiceExt::<Reconnect>::call_all(self.as_service_mut(), iter([Reconnect, Reconnect]))
            .fold(Ok(()), move |res, x| match state {
                State::Init => {
                    state = State::Reconnect;
                    match x {
                        Ok(()) => ready(Ok(())),
                        Err(err) => ready(Err(ExchangeError::layer(err))),
                    }
                }
                State::Reconnect => ready(res),
            })
            .boxed()
    }
}
