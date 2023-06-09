use crate::{types::Name, websocket::protocol::frame::mini_ticker::Statistic, Request};
use exc_core::{types::ticker as types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};
use time::OffsetDateTime;

impl Adaptor<types::SubscribeStatistics> for Request {
    fn from_request(req: types::SubscribeStatistics) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe(Name::mini_ticker(&req.instrument)))
    }

    fn into_response(resp: Self::Response) -> Result<types::StatisticStream, ExchangeError> {
        let stream = resp.into_stream::<Statistic>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|t| async move {
                Ok(types::Statistic {
                    ts: t
                        .event_timestamp
                        .map(super::from_timestamp)
                        .unwrap_or_else(|| Ok(OffsetDateTime::now_utc()))?,
                    close: t.close.normalize(),
                    open: t.open.normalize(),
                    high: t.high.normalize(),
                    low: t.low.normalize(),
                    vol: t.vol.normalize(),
                })
            })
            .boxed())
    }
}
