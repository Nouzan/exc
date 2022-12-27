use std::{collections::BTreeMap, sync::Arc};

use crate::core::{types::instrument::InstrumentMeta, Str, Symbol};
use either::Either;
use rust_decimal::Decimal;

#[derive(Default)]
pub(super) struct InstState {
    insts: BTreeMap<Symbol, Arc<InstrumentMeta<Decimal>>>,
    alias: BTreeMap<Str, Symbol>,
}

impl InstState {
    pub(super) fn get(&self, inst: &Either<Symbol, Str>) -> Option<&Arc<InstrumentMeta<Decimal>>> {
        let symbol = inst.as_ref().either(Some, |name| self.alias.get(name))?;
        self.insts.get(symbol)
    }

    pub(super) fn insert(&mut self, inst: InstrumentMeta<Decimal>) {
        let name = inst.smol_name().clone();
        let symbol = inst.instrument().as_symbol().clone();
        tracing::debug!(%name, %symbol, "new binding");
        self.alias.insert(name, symbol.clone());
        self.insts.insert(symbol, Arc::new(inst));
    }
}
