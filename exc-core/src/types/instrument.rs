use derive_more::Display;
use futures::stream::BoxStream;
use positions::Instrument;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ExchangeError, Request, Str};

/// Parse Instrument Meta Error.
#[derive(Debug, Error)]
pub enum InstrumentMetaError<E> {
    /// Parse num error.
    #[error("parse num error: {0}")]
    FromStrError(#[from] E),

    /// Missing fields.
    #[error("missing fields")]
    MissingFields,
}

/// Instrument Meta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[display(bound = "Num: std::fmt::Display")]
#[display(
    fmt = "inst={inst} rev={} unit={unit} pt={price_tick} st={size_tick} ms={min_size} mv={min_value}",
    "inst.is_prefer_reversed()"
)]
#[serde(bound = "Num: num_traits::Zero + Serialize + for<'a> Deserialize<'a>")]
pub struct InstrumentMeta<Num> {
    /// Instrument.
    pub inst: Instrument,
    /// Unit.
    pub unit: Num,
    /// Price min tick.
    pub price_tick: Num,
    /// Size min tick.
    pub size_tick: Num,
    /// Min trade size.
    pub min_size: Num,
    /// Min value.
    #[serde(default = "num_traits::Zero::zero")]
    pub min_value: Num,
}

/// Instrument Stream.
pub type InstrumentStream = BoxStream<'static, Result<InstrumentMeta<Decimal>, ExchangeError>>;

/// Subscribe instruments.
#[derive(Debug, Clone)]
pub struct SubscribeInstruments {
    /// Tag.
    pub tag: Str,
}

/// Fetch instruments.
#[derive(Debug, Clone)]
pub struct FetchInstruments {
    /// Tag.
    pub tag: Str,
}

impl Request for SubscribeInstruments {
    type Response = InstrumentStream;
}

impl Request for FetchInstruments {
    type Response = InstrumentStream;
}
