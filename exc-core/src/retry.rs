use std::marker::PhantomData;

use futures::{future::BoxFuture, FutureExt};
use tower::retry::Policy;

const DEFAULT_MAX_SECS_TO_WAIT: u64 = 128;

/// Retry Policy.
#[derive(Debug)]
pub enum RetryPolicy<T, U, F = ()> {
    /// On.
    On {
        /// Error filter.
        f: F,
        /// Retry times.
        times: usize,
        /// Max secs to wait.
        max_secs: u64,
    },
    /// Never.
    Never(PhantomData<fn() -> (T, U)>),
}

impl<T, U, F: Clone> Clone for RetryPolicy<T, U, F> {
    fn clone(&self) -> Self {
        match self {
            Self::Never(_) => Self::Never(PhantomData),
            Self::On { f, times, max_secs } => Self::On {
                f: f.clone(),
                times: *times,
                max_secs: *max_secs,
            },
        }
    }
}

impl<T, U, F: Copy> Copy for RetryPolicy<T, U, F> {}

impl<T, U> Default for RetryPolicy<T, U, ()> {
    fn default() -> Self {
        Self::never()
    }
}

impl<T, U, F> RetryPolicy<T, U, F> {
    /// Never retry.
    pub fn never() -> Self {
        Self::Never(PhantomData)
    }

    /// Retry on.
    pub fn retry_on<E, F2>(self, f: F2) -> RetryPolicy<T, U, F2>
    where
        F2: Fn(&E) -> bool,
        F2: Send + 'static + Clone,
    {
        RetryPolicy::On {
            f,
            times: 0,
            max_secs: DEFAULT_MAX_SECS_TO_WAIT,
        }
    }

    /// Retry on with max wait secs.
    pub fn retry_on_with_max_wait_secs<E, F2>(self, f: F2, secs: u64) -> RetryPolicy<T, U, F2>
    where
        F2: Fn(&E) -> bool,
        F2: Send + 'static + Clone,
    {
        RetryPolicy::On {
            f,
            times: 0,
            max_secs: secs,
        }
    }
}

impl<T, U, E, F> Policy<T, U, E> for RetryPolicy<T, U, F>
where
    T: 'static + Clone,
    U: 'static,
    F: Fn(&E) -> bool,
    F: Send + 'static + Clone,
{
    type Future = BoxFuture<'static, Self>;

    fn retry(&self, _req: &T, result: Result<&U, &E>) -> Option<Self::Future> {
        match self {
            Self::On { f, times, max_secs } => match result {
                Ok(_) => None,
                Err(err) => {
                    if f(err) {
                        let times = *times;
                        let secs = (1 << times).min(*max_secs);
                        tracing::trace!("retry in {secs}s;");
                        let retry = Self::On {
                            f: f.clone(),
                            times: times + 1,
                            max_secs: *max_secs,
                        };
                        let fut = async move {
                            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                            retry
                        }
                        .boxed();
                        Some(fut)
                    } else {
                        tracing::trace!("retry given up;");
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
