use exc_core::{types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};
use time::OffsetDateTime;

use crate::{
    websocket::{protocol::frame::book_ticker::BookTicker, request::WsRequest},
    Request,
};

impl Adaptor<types::SubscribeBidAsk> for Request {
    fn from_request(req: types::SubscribeBidAsk) -> Result<Self, ExchangeError> {
        Ok(WsRequest::dispatch_bid_ask(req).into())
    }

    fn into_response(resp: Self::Response) -> Result<types::BidAskStream, ExchangeError> {
        let stream = resp.into_stream::<BookTicker>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|t| async move {
                Ok(types::BidAsk {
                    ts: t
                        .trade_timestamp
                        .map(super::from_timestamp)
                        .unwrap_or_else(|| Ok(OffsetDateTime::now_utc()))?,
                    bid: Some((t.bid.normalize(), t.bid_size.normalize())),
                    ask: Some((t.ask.normalize(), t.ask_size.normalize())),
                })
            })
            .boxed())
    }
}
