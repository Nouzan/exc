use exc_core::{types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};

use crate::{
    websocket::{protocol::frame::TradeFrame, request::WsRequest},
    Request,
};

impl Adaptor<types::SubscribeTrades> for Request {
    fn from_request(req: types::SubscribeTrades) -> Result<Self, ExchangeError> {
        Ok(WsRequest::dispatch_trades(req).into())
    }

    fn into_response(resp: Self::Response) -> Result<types::TradeStream, ExchangeError> {
        let stream = resp.into_stream::<TradeFrame>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|trade| async move { trade.try_into() })
            .boxed())
    }
}
