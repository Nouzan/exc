use std::sync::Arc;

use exc_core::{ExchangeError, Str};

use super::{state::State, InstrumentSvc};

pub(super) struct Worker {
    state: Arc<State>,
    inst: InstrumentSvc,
}

impl Worker {
    pub(super) fn new(state: &Arc<State>, inst: InstrumentSvc) -> Self {
        Self {
            state: state.clone(),
            inst,
        }
    }

    pub(super) async fn start(self) -> Result<(), ExchangeError> {
        let Self { state, inst } = self;
        let inst = state.watch_instruments(inst, Str::new("SPOT"));
        tokio::select! {
            res = inst => {
                res?;
            }
        }
        Ok(())
    }
}
