use exc_core::{types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};

use crate::{types::Name, websocket::protocol::frame::agg_trade::AggTrade, Request};

impl Adaptor<types::SubscribeTrades> for Request {
    fn from_request(req: types::SubscribeTrades) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe(Name::agg_trade(&req.instrument)))
    }

    fn into_response(resp: Self::Response) -> Result<types::TradeStream, ExchangeError> {
        let stream = resp.into_stream::<AggTrade>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|trade| async move {
                Ok(types::Trade {
                    ts: super::from_timestamp(trade.trade_timestamp)?,
                    price: trade.price,
                    size: trade.size,
                    buy: !trade.buy_maker,
                })
            })
            .boxed())
    }
}
