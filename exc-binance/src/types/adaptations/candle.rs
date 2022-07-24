use crate::{
    http::{request, response},
    Request,
};
use exc_core::{types, Adaptor, ExchangeError};
use futures::StreamExt;
use std::{ops::RangeBounds, time::Duration};
use time::UtcOffset;

const M1_SECS: u64 = 60;
const H1_SECS: u64 = M1_SECS * 60;
const D1_SECS: u64 = H1_SECS * 24;

const M1: Duration = Duration::from_secs(M1_SECS);
const M3: Duration = Duration::from_secs(M1_SECS * 3);
const M5: Duration = Duration::from_secs(M1_SECS * 5);
const M15: Duration = Duration::from_secs(M1_SECS * 15);
const M30: Duration = Duration::from_secs(M1_SECS * 30);
const H1: Duration = Duration::from_secs(H1_SECS);
const H2: Duration = Duration::from_secs(H1_SECS * 2);
const H4: Duration = Duration::from_secs(H1_SECS * 4);
const H6: Duration = Duration::from_secs(H1_SECS * 6);
const H8: Duration = Duration::from_secs(H1_SECS * 8);
const H12: Duration = Duration::from_secs(H1_SECS * 12);
const D1: Duration = Duration::from_secs(D1_SECS);
const D3: Duration = Duration::from_secs(D1_SECS * 3);
const W1: Duration = Duration::from_secs(D1_SECS * 7);

impl TryFrom<types::Period> for request::Interval {
    type Error = ExchangeError;

    fn try_from(period: types::Period) -> Result<Self, Self::Error> {
        if period.utc_offset() != UtcOffset::UTC {
            return Err(ExchangeError::Other(anyhow!(
                "unsupported timezone: {}",
                period.utc_offset()
            )));
        }
        match period.kind() {
            types::PeriodKind::Year => Err(ExchangeError::Other(anyhow!(
                "unsupported period: {}",
                period
            ))),
            types::PeriodKind::Month => Ok(Self::M1),
            types::PeriodKind::Duration(dur) => match dur {
                M1 => Ok(Self::M1),
                M3 => Ok(Self::M3),
                M5 => Ok(Self::M5),
                M15 => Ok(Self::M15),
                M30 => Ok(Self::M30),
                H1 => Ok(Self::H1),
                H2 => Ok(Self::H2),
                H4 => Ok(Self::H4),
                H6 => Ok(Self::H6),
                H8 => Ok(Self::H8),
                H12 => Ok(Self::H6),
                D1 => Ok(Self::D1),
                D3 => Ok(Self::D3),
                W1 => Ok(Self::W1),
                _ => Err(ExchangeError::Other(anyhow!(
                    "unsupported period: {}",
                    period
                ))),
            },
        }
    }
}

impl Adaptor<types::QueryFirstCandles> for Request {
    fn from_request(req: types::QueryFirstCandles) -> Result<Self, ExchangeError> {
        Ok(Request::with_rest_payload(request::QueryCandles {
            symbol: req.query().inst().to_uppercase(),
            interval: req.query().period().try_into()?,
            start_time: super::start_bound_to_timestamp(req.query().start_bound())?,
            end_time: super::end_bound_to_timestamp(req.query().end_bound())?,
            limit: Some(req.first()),
        }))
    }

    fn into_response(resp: Self::Response) -> Result<types::CandleStream, ExchangeError> {
        let candles = resp
            .into_response::<response::Candles>()?
            .into_iter()
            .map(|c| {
                Ok(types::Candle {
                    ts: super::from_timestamp(c.0)?,
                    open: c.1,
                    high: c.2,
                    low: c.3,
                    close: c.4,
                    volume: c.5,
                })
            });
        Ok(futures::stream::iter(candles).boxed())
    }
}
