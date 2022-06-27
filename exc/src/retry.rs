use std::marker::PhantomData;

use futures::{future::BoxFuture, FutureExt};
use tower::retry::Policy;

use crate::ExchangeError;

/// Retry Policy.
#[derive(Debug)]
pub enum RetryPolicy<T, U, F> {
    /// On.
    On {
        /// Error filter.
        f: F,
        /// Retry times.
        times: usize,
    },
    /// Never.
    Never(PhantomData<fn() -> (T, U)>),
}

impl<T, U, F: Clone> Clone for RetryPolicy<T, U, F> {
    fn clone(&self) -> Self {
        match self {
            Self::Never(_) => Self::Never(PhantomData),
            Self::On { f, times } => Self::On {
                f: f.clone(),
                times: *times,
            },
        }
    }
}

impl<T, U, F: Copy> Copy for RetryPolicy<T, U, F> {}

impl<T, U, F> RetryPolicy<T, U, F> {
    /// Never retry.
    pub fn never() -> Self {
        Self::Never(PhantomData)
    }

    /// Retry on.
    pub fn retry_on<F2>(self, f: F2) -> RetryPolicy<T, U, F2>
    where
        F2: Fn(&ExchangeError) -> bool,
        F2: Send + 'static + Clone,
    {
        RetryPolicy::On { f, times: 0 }
    }
}

impl<T, U, F> Policy<T, U, ExchangeError> for RetryPolicy<T, U, F>
where
    T: 'static + Clone,
    U: 'static,
    F: Fn(&ExchangeError) -> bool,
    F: Send + 'static + Clone,
{
    type Future = BoxFuture<'static, Self>;

    fn retry(&self, _req: &T, result: Result<&U, &ExchangeError>) -> Option<Self::Future> {
        match self {
            Self::On { f, times } => match result {
                Ok(_) => None,
                Err(err) => {
                    if f(err) {
                        let times = *times;
                        let secs = (1 << times).min(128);
                        tracing::trace!("retry in {secs}s; err={err}");
                        let retry = Self::On {
                            f: f.clone(),
                            times: times + 1,
                        };
                        let fut = async move {
                            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                            retry
                        }
                        .boxed();
                        Some(fut)
                    } else {
                        tracing::trace!("retry given up; err={err}");
                        None
                    }
                }
            },
            Self::Never(_) => None,
        }
    }

    fn clone_request(&self, req: &T) -> Option<T> {
        Some(req.clone())
    }
}
