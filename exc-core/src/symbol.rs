use positions::{prelude::Str, Asset, ParseSymbolError};
use std::{fmt, str::FromStr};
use thiserror::Error;
use time::{macros::format_description, parsing::Parsable, Date};

use crate::Symbol;

/// The exc format symbol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExcSymbol(Symbol);

impl AsRef<Symbol> for ExcSymbol {
    fn as_ref(&self) -> &Symbol {
        &self.0
    }
}

/// Options Type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionsType {
    /// Put.
    Put(Str),
    /// Call.
    Call(Str),
}

/// Symbol Type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    /// Spot.
    Spot,
    /// Margin.
    Margin,
    /// Futures.
    Futures(Str),
    /// Perpetual.
    Perpetual,
    /// Options.
    Options(Str, OptionsType),
}

impl ExcSymbol {
    /// Margin tag.
    pub const MARGIN: &str = "";
    /// Futures tag.
    pub const FUTURES: &str = "F";
    /// Perpetual tag.
    pub const PERPETUAL: &str = "P";
    /// Options tag.
    pub const OPTIONS: &str = "O";
    /// Put options tag.
    pub const PUT: &str = "P";
    /// Call options tag.
    pub const CALL: &str = "C";
    /// Seperate tag.
    pub const SEP: char = '-';

    /// Get date format.
    pub fn date_format() -> impl Parsable {
        format_description!("[year][month][day]")
    }

    /// Exchange.
    pub fn exchange(&self) -> Option<&str> {
        let (prefix, _) = self.0.as_derivative()?;
        Some(prefix)
    }

    /// Try creating from symbol.
    pub fn new(symbol: &Symbol) -> Option<Self> {
        if symbol.is_spot() {
            Some(Self(symbol.clone()))
        } else if let Some((_, sym)) = symbol.as_derivative() {
            if !sym.is_ascii() {
                return None;
            }
            if let Some(extra) = sym.split(Self::SEP).nth(2) {
                let (ty, extra) = extra.split_at(1);
                match ty {
                    Self::FUTURES => {
                        let format = Self::date_format();
                        Date::parse(&format!("20{}", extra), &format).ok()?;
                    }
                    Self::PERPETUAL => {}
                    Self::OPTIONS => {
                        if extra.len() <= 7 {
                            return None;
                        }
                        let (date, opts) = extra.split_at(6);
                        let format = Self::date_format();
                        Date::parse(&format!("20{}", date), &format).ok()?;
                        let (opts, _price) = opts.split_at(1);
                        match opts {
                            Self::PUT => {}
                            Self::CALL => {}
                            _ => return None,
                        };
                    }
                    _ => {
                        return None;
                    }
                }
            }
            Some(Self(symbol.clone()))
        } else {
            None
        }
    }

    /// Divide symbol into parts: `(base, quote, type)`.
    pub fn to_parts(&self) -> (Asset, Asset, SymbolType) {
        if let Some((base, quote)) = self.0.as_spot() {
            (base.clone(), quote.clone(), SymbolType::Spot)
        } else if let Some((_, symbol)) = self.0.as_derivative() {
            let mut parts = symbol.split(Self::SEP);
            let base = parts.next().unwrap();
            let quote = parts.next().unwrap();
            let ty = if let Some(extra) = parts.next() {
                let (ty, extra) = extra.split_at(1);
                match ty {
                    Self::FUTURES => {
                        debug_assert_eq!(extra.len(), 6);
                        SymbolType::Futures(Str::new_inline(extra))
                    }
                    Self::PERPETUAL => SymbolType::Perpetual,
                    Self::OPTIONS => {
                        let (date, opts) = extra.split_at(6);
                        let (opts, price) = opts.split_at(1);
                        let opts = match opts {
                            Self::PUT => OptionsType::Put(Str::new(price)),
                            Self::CALL => OptionsType::Call(Str::new(price)),
                            _ => unreachable!(),
                        };
                        SymbolType::Options(Str::new_inline(date), opts)
                    }
                    _ => unreachable!(),
                }
            } else {
                SymbolType::Margin
            };
            (
                Asset::from_str(base).unwrap(),
                Asset::from_str(quote).unwrap(),
                ty,
            )
        } else {
            unreachable!()
        }
    }
}

impl fmt::Display for ExcSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Parse [`ExcSymbol`] Error.
#[derive(Debug, Error)]
pub enum ParseExcSymbolError {
    /// Parse symbol error.
    #[error("parse symbol error: {0}")]
    ParseSymbol(#[from] ParseSymbolError),
    /// Invalid format.
    #[error("invalid format")]
    InvalidFormat,
}

impl FromStr for ExcSymbol {
    type Err = ParseExcSymbolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let symbol = Symbol::from_str(s)?;
        Self::new(&symbol).ok_or(ParseExcSymbolError::InvalidFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot() {
        let symbol: ExcSymbol = "BTC-USDT".parse().unwrap();
        assert!(symbol.0.is_spot());
        assert_eq!(
            symbol.to_parts(),
            (Asset::BTC, Asset::USDT, SymbolType::Spot)
        );
    }

    #[test]
    fn test_margin() {
        let symbol: ExcSymbol = ":BTC-USDT".parse().unwrap();
        assert!(!symbol.0.is_spot());
        assert_eq!(
            symbol.to_parts(),
            (Asset::BTC, Asset::USDT, SymbolType::Margin)
        );
    }

    #[test]
    fn test_futures() {
        let symbol: ExcSymbol = ":BTC-USDT-F221230".parse().unwrap();
        assert!(!symbol.0.is_spot());
        assert_eq!(
            symbol.to_parts(),
            (
                Asset::BTC,
                Asset::USDT,
                SymbolType::Futures(Str::new("221230"))
            )
        );
    }

    #[test]
    fn test_perpetual() {
        let symbol: ExcSymbol = ":BTC-USDT-P".parse().unwrap();
        assert!(!symbol.0.is_spot());
        assert_eq!(
            symbol.to_parts(),
            (Asset::BTC, Asset::USDT, SymbolType::Perpetual,)
        );
    }

    #[test]
    fn test_call_options() {
        let symbol: ExcSymbol = ":BTC-USDT-O221230C17000".parse().unwrap();
        assert!(!symbol.0.is_spot());
        assert_eq!(
            symbol.to_parts(),
            (
                Asset::BTC,
                Asset::USDT,
                SymbolType::Options(Str::new("221230"), OptionsType::Call(Str::new("17000"))),
            )
        );
    }

    #[test]
    fn test_put_options() {
        let symbol: ExcSymbol = ":BTC-USDT-O221230P17000".parse().unwrap();
        assert!(!symbol.0.is_spot());
        assert_eq!(
            symbol.to_parts(),
            (
                Asset::BTC,
                Asset::USDT,
                SymbolType::Options(Str::new("221230"), OptionsType::Put(Str::new("17000"))),
            )
        );
    }
}
