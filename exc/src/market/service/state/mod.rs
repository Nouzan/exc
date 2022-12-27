use std::sync::{Arc, RwLock};

use exc_core::{
    types::instrument::{FetchInstruments, InstrumentMeta, SubscribeInstruments},
    ExchangeError, Str,
};
use futures::{stream, StreamExt, TryStreamExt};
use rust_decimal::Decimal;
use tower::ServiceExt;

use crate::types::instrument::GetInstrument;

use super::{FetchInstrumentSvc, SubscribeInstrumentSvc};

mod inst;

#[derive(Default)]
pub(super) struct State {
    insts: RwLock<inst::InstState>,
}

impl State {
    pub(super) async fn init(
        self: Arc<Self>,
        mut fetch: FetchInstrumentSvc,
        tags: Vec<Str>,
    ) -> Result<(), ExchangeError> {
        let mut finished = false;
        while !finished {
            let mut stream = fetch
                .ready()
                .await?
                .call_all(stream::iter(tags.iter().cloned()).map(|tag| FetchInstruments { tag }))
                .boxed()
                .try_flatten();
            while let Some(meta) = stream.next().await {
                match meta {
                    Ok(meta) => {
                        self.insts.write().unwrap().insert(meta);
                    }
                    Err(err) => {
                        tracing::error!(%err, "init; fetch instruments stream error");
                        break;
                    }
                }
            }
            finished = true;
        }
        Ok(())
    }

    pub(super) fn get_instrument(
        &self,
        req: &GetInstrument,
    ) -> Option<Arc<InstrumentMeta<Decimal>>> {
        tracing::debug!(symbol=%req.symbol, "getting instrument");
        self.insts.read().unwrap().get(&req.symbol).cloned()
    }

    pub(super) async fn watch_instruments(
        self: Arc<Self>,
        mut svc: SubscribeInstrumentSvc,
        tags: Vec<Str>,
    ) -> Result<(), ExchangeError> {
        loop {
            let mut stream = svc
                .ready()
                .await?
                .call_all(stream::iter(
                    tags.iter().cloned().map(|tag| SubscribeInstruments { tag }),
                ))
                .boxed()
                .try_flatten();
            while let Some(meta) = stream.next().await {
                match meta {
                    Ok(meta) => {
                        self.insts.write().unwrap().insert(meta);
                    }
                    Err(err) => {
                        tracing::error!(%err, "watch instruments; stream error");
                        break;
                    }
                }
            }
        }
    }
}
