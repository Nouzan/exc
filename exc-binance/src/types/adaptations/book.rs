use exc_core::{types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};

use crate::{types::Name, websocket::protocol::frame::book_ticker::BookTicker, Request};

impl Adaptor<types::SubscribeBidAsk> for Request {
    fn from_request(req: types::SubscribeBidAsk) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe(Name::book_ticker(&req.instrument)))
    }

    fn into_response(resp: Self::Response) -> Result<types::BidAskStream, ExchangeError> {
        let stream = resp.into_stream::<BookTicker>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|t| async move {
                Ok(types::BidAsk {
                    ts: super::from_timestamp(t.trade_timestamp)?,
                    bid: Some((t.bid, t.bid_size)),
                    ask: Some((t.ask, t.ask_size)),
                })
            })
            .boxed())
    }
}
