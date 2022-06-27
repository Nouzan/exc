use exc_core::{
    types::{
        trading::{GetOrder, Order as ExcOrder, OrderId, OrderState, OrderStatus, Place},
        Adaptor,
    },
    ExchangeError,
};
use futures::FutureExt;
use rust_decimal::Decimal;

use crate::http::types::{
    request::{trading::Order, HttpRequest, PrivateGet},
    response::ResponseData,
};

impl Adaptor<GetOrder> for HttpRequest {
    fn from_request(req: GetOrder) -> Result<Self, exc_core::ExchangeError>
    where
        Self: Sized,
    {
        Ok(HttpRequest::PrivateGet(PrivateGet::Order(Order {
            inst_id: req.instrument,
            ord_id: Some(req.id.as_str().to_string()),
            cl_ord_id: None,
        })))
    }

    fn into_response(
        mut resp: Self::Response,
    ) -> Result<<GetOrder as exc_core::types::Request>::Response, exc_core::ExchangeError> {
        Ok(async move {
            if let Some(data) = resp.data.pop() {
                if let ResponseData::Order(order) = data {
                    let (target, buy) = match order.side.as_str() {
                        "buy" => (Place::with_size(order.size), true),
                        "sell" => (Place::with_size(-order.size), false),
                        side => {
                            return Err(ExchangeError::Other(anyhow::anyhow!(
                                "unexpected order side: {side}"
                            )));
                        }
                    };
                    let target = match order.order_type.as_str() {
                        "market" => target,
                        "limit" => {
                            if let Some(price) = order.price {
                                target.limit(price)
                            } else {
                                return Err(ExchangeError::Other(anyhow::anyhow!(
                                    "limit without price"
                                )));
                            }
                        }
                        t => {
                            return Err(ExchangeError::Other(anyhow::anyhow!(
                                "unsupported order type: {t}"
                            )));
                        }
                    };
                    let mut state = OrderState::default();
                    let status = match order.state.as_str() {
                        "live" | "partially_filled" => OrderStatus::Pending,
                        "canceled" | "filled" => OrderStatus::Finished,
                        s => {
                            return Err(ExchangeError::Other(anyhow::anyhow!(
                                "unknown order status: {s}"
                            )))
                        }
                    };
                    let mut filled = order.filled_size;
                    filled.set_sign_positive(buy);
                    let cost = order.avg_price.unwrap_or(Decimal::ONE);
                    if let Some((ccy, fee)) = order
                        .fee
                        .and_then(|fee| order.fee_currency.map(|ccy| (ccy, fee)))
                    {
                        let f = state.fees.entry(ccy).or_default();
                        *f += fee;
                    }
                    if let Some((ccy, fee)) = order
                        .rebate
                        .and_then(|fee| order.rebate_currency.map(|ccy| (ccy, fee)))
                    {
                        let f = state.fees.entry(ccy).or_default();
                        *f += fee;
                    }
                    state.status = status;
                    state.filled = filled;
                    state.cost = cost;
                    Ok(ExcOrder {
                        id: OrderId::from(order.order_id),
                        target,
                        state,
                    })
                } else {
                    Err(ExchangeError::Api(anyhow::anyhow!(
                        "unexpected response type"
                    )))
                }
            } else {
                Err(ExchangeError::Api(anyhow::anyhow!("empty response")))
            }
        }
        .boxed())
    }
}
