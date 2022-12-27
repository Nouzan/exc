use std::sync::{Arc, RwLock};

use exc_core::{
    types::instrument::{FetchInstruments, InstrumentMeta, SubscribeInstruments},
    ExchangeError, Str,
};
use futures::StreamExt;
use rust_decimal::Decimal;
use tower::{Service, ServiceExt};

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
        tag: Str,
    ) -> Result<(), ExchangeError> {
        let mut finished = false;
        while !finished {
            let mut stream = fetch
                .ready()
                .await?
                .call(FetchInstruments { tag: tag.clone() })
                .await?;
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
        tag: Str,
    ) -> Result<(), ExchangeError> {
        loop {
            let mut stream = svc
                .ready()
                .await?
                .call(SubscribeInstruments { tag: tag.clone() })
                .await?;
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
