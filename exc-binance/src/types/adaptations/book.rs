use exc_core::{types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};

use crate::{
    websocket::{protocol::frame::DepthFrame, request::WsRequest},
    Request,
};

impl Adaptor<types::SubscribeBidAsk> for Request {
    fn from_request(req: types::SubscribeBidAsk) -> Result<Self, ExchangeError> {
        Ok(WsRequest::dispatch_bid_ask(req).into())
    }

    fn into_response(resp: Self::Response) -> Result<types::BidAskStream, ExchangeError> {
        let stream = resp.into_stream::<DepthFrame>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|t| async move { t.try_into() })
            .boxed())
    }
}
