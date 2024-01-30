use derive_more::Display;
use futures::stream::BoxStream;
use positions::Instrument;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{symbol::ExcSymbol, Str};
use exc_service::{ExchangeError, Request};

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
    fmt = "name={name} inst={inst} rev={} unit={} pt={} st={} ms={} mv={} live={live} will_expire={}",
    "inst.is_prefer_reversed()",
    "attrs.unit",
    "attrs.price_tick",
    "attrs.size_tick",
    "attrs.min_size",
    "attrs.min_value",
    "expire.is_some()"
)]
#[serde(bound = "Num: num_traits::Zero + Serialize + for<'a> Deserialize<'a>")]
pub struct InstrumentMeta<Num> {
    /// name.
    name: Str,
    /// Instrument.
    inst: Instrument,
    /// Attributes.
    #[serde(flatten)]
    attrs: Attributes<Num>,
    /// Is live for trading.
    #[serde(default = "live")]
    live: bool,
    /// Expire time.
    #[serde(default)]
    #[serde(with = "time::serde::rfc3339::option")]
    expire: Option<time::OffsetDateTime>,
}

fn live() -> bool {
    true
}

/// Instrument Meta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "Num: num_traits::Zero + Serialize + for<'a> Deserialize<'a>")]
pub struct Attributes<Num> {
    /// Reversed.
    pub reversed: bool,
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

impl<Num> InstrumentMeta<Num> {
    /// Create a new [`InstrumentMeta`].
    pub fn new(name: impl AsRef<str>, symbol: ExcSymbol, attrs: Attributes<Num>) -> Self {
        let (base, quote, _) = symbol.to_parts();
        let inst = Instrument::try_with_symbol(symbol.into(), &base, &quote)
            .expect("symbol must be valid")
            .prefer_reversed(attrs.reversed);
        Self {
            name: Str::new(name),
            inst,
            attrs,
            live: true,
            expire: None,
        }
    }

    /// Get the instrument name from the exchange.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get name of inner type.
    pub fn smol_name(&self) -> &Str {
        &self.name
    }

    /// Get instrument.
    pub fn instrument(&self) -> &Instrument {
        &self.inst
    }

    /// Get attributes.
    pub fn attrs(&self) -> &Attributes<Num> {
        &self.attrs
    }

    /// Get if the instrument is live for trading.
    pub fn is_live(&self) -> bool {
        self.live
    }

    /// Get the expire time.
    pub fn expire(&self) -> Option<&time::OffsetDateTime> {
        self.expire.as_ref()
    }

    /// Set whether the instrument is live for trading.
    pub fn with_live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }

    /// Set the expire time.
    pub fn with_expire(mut self, expire: impl Into<Option<time::OffsetDateTime>>) -> Self {
        self.expire = expire.into();
        self
    }
}

/// Instrument Stream.
pub type InstrumentStream = BoxStream<'static, Result<InstrumentMeta<Decimal>, ExchangeError>>;

/// Subscribe instruments.
#[derive(Debug, Clone)]
pub struct SubscribeInstruments {
    /// Tag.
    pub tag: Str,
}

impl SubscribeInstruments {
    /// Create a new [`SubscribeInstruments`] request.
    pub fn new(tag: impl AsRef<str>) -> Self {
        Self { tag: Str::new(tag) }
    }
}

/// Fetch instruments.
#[derive(Debug, Clone)]
pub struct FetchInstruments {
    /// Tag.
    pub tag: Str,
}

impl FetchInstruments {
    /// Create a new [`FetchInstruments`] request.
    pub fn new(tag: impl AsRef<str>) -> Self {
        Self { tag: Str::new(tag) }
    }
}

impl Request for SubscribeInstruments {
    type Response = InstrumentStream;
}

impl Request for FetchInstruments {
    type Response = InstrumentStream;
}
