use exc_core::{types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};

use crate::{
    websocket::{protocol::frame::agg_trade::AggTrade, request::WsRequest},
    Request,
};

impl Adaptor<types::SubscribeTrades> for Request {
    fn from_request(req: types::SubscribeTrades) -> Result<Self, ExchangeError> {
        Ok(WsRequest::dispatch_trades(req).into())
    }

    fn into_response(resp: Self::Response) -> Result<types::TradeStream, ExchangeError> {
        let stream = resp.into_stream::<AggTrade>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|trade| async move {
                Ok(types::Trade {
                    ts: super::from_timestamp(trade.trade_timestamp)?,
                    price: trade.price.normalize(),
                    size: trade.size.normalize(),
                    buy: !trade.buy_maker,
                })
            })
            .boxed())
    }
}
