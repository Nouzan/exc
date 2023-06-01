use crate::{types::Name, websocket::protocol::frame::mini_ticker::MiniTicker, Request};
use exc_core::{types::ticker as types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};
use time::OffsetDateTime;

impl Adaptor<types::SubscribeMiniTickers> for Request {
    fn from_request(req: types::SubscribeMiniTickers) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe(Name::mini_ticker(&req.instrument)))
    }

    fn into_response(resp: Self::Response) -> Result<types::MiniTickerStream, ExchangeError> {
        let stream = resp.into_stream::<MiniTicker>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|t| async move {
                Ok(types::MiniTicker {
                    ts: t
                        .event_timestamp
                        .map(super::from_timestamp)
                        .unwrap_or_else(|| Ok(OffsetDateTime::now_utc()))?,
                    last: t.close.normalize(),
                    open_24h: t.open.normalize(),
                    high_24h: t.high.normalize(),
                    low_24h: t.low.normalize(),
                    vol_24h: t.vol.normalize(),
                    vol_quote_24h: t.vol_quote.normalize(),
                })
            })
            .boxed())
    }
}
