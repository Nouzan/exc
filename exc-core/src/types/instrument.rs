use derive_more::Display;
use futures::stream::BoxStream;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ExchangeError, Request};

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
    fmt = "name={name} r={is_reversed} unit={unit} pt={price_tick} st={size_tick} ms={min_size} mv={min_value}"
)]
pub struct InstrumentMeta<Num> {
    /// Name.
    pub name: String,
    /// Is reversed price representation.
    pub is_reversed: bool,
    /// Unit.
    pub unit: Num,
    /// Price min tick.
    pub price_tick: Num,
    /// Size min tick.
    pub size_tick: Num,
    /// Min trade size.
    pub min_size: Num,
    /// Min value.
    pub min_value: Num,
}

impl<Num: num_traits::Num> InstrumentMeta<Num> {
    /// Parse from str by split.
    pub fn from_str_by_split(
        s: &str,
        pat: char,
        reversed_str: &str,
    ) -> Result<Self, InstrumentMetaError<Num::FromStrRadixErr>> {
        let mut splited = s.split(pat);
        let name = splited
            .next()
            .ok_or(InstrumentMetaError::MissingFields)?
            .to_string();
        let is_reversed = splited.next().ok_or(InstrumentMetaError::MissingFields)? == reversed_str;
        let unit = Num::from_str_radix(
            splited.next().ok_or(InstrumentMetaError::MissingFields)?,
            10,
        )?;
        let price_tick = Num::from_str_radix(
            splited.next().ok_or(InstrumentMetaError::MissingFields)?,
            10,
        )?;
        let size_tick = Num::from_str_radix(
            splited.next().ok_or(InstrumentMetaError::MissingFields)?,
            10,
        )?;
        let min_size = Num::from_str_radix(
            splited.next().ok_or(InstrumentMetaError::MissingFields)?,
            10,
        )?;
        Ok(Self {
            name,
            is_reversed,
            unit,
            price_tick,
            size_tick,
            min_size,
            min_value: Num::zero(),
        })
    }
}

/// Instrument Stream.
pub type InstrumentStream = BoxStream<'static, Result<InstrumentMeta<Decimal>, ExchangeError>>;

/// Subscribe instruments.
#[derive(Debug, Clone)]
pub struct SubscribeInstruments {
    /// Tag.
    pub tag: String,
}

/// Fetch instruments.
#[derive(Debug, Clone)]
pub struct FetchInstruments {
    /// Tag.
    pub tag: String,
}

impl Request for SubscribeInstruments {
    type Response = InstrumentStream;
}

impl Request for FetchInstruments {
    type Response = InstrumentStream;
}
