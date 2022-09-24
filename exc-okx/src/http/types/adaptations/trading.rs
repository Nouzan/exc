use exc_core::{
    types::{
        trading::{GetOrder, Order as ExcOrder, OrderId, OrderState, OrderStatus, Place},
        OrderUpdate, TimeInForce,
    },
    Adaptor, ExchangeError,
};
use futures::FutureExt;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use time::OffsetDateTime;

use crate::{
    http::types::{
        request::{trading::Order, HttpRequest, PrivateGet},
        response::ResponseData,
    },
    util::timestamp::millis_to_ts,
};

fn decimal_to_ts(ts: Decimal) -> Option<OffsetDateTime> {
    millis_to_ts(ts.to_u64()?)
}

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
    ) -> Result<<GetOrder as exc_core::Request>::Response, exc_core::ExchangeError> {
        Ok(async move {
            if let Some(data) = resp.data.pop() {
                if let ResponseData::Order(order) = data {
                    let order = *order;
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
                        "fok" => {
                            if let Some(price) = order.price {
                                target.limit_with_tif(price, TimeInForce::FillOrKill)
                            } else {
                                return Err(ExchangeError::Other(anyhow::anyhow!(
                                    "fok without price"
                                )));
                            }
                        }
                        "ioc" => {
                            if let Some(price) = order.price {
                                target.limit_with_tif(price, TimeInForce::ImmediateOrCancel)
                            } else {
                                return Err(ExchangeError::Other(anyhow::anyhow!(
                                    "ioc without price"
                                )));
                            }
                        }
                        "post_only" => {
                            if let Some(price) = order.price {
                                target.post_only(price)
                            } else {
                                return Err(ExchangeError::Other(anyhow::anyhow!(
                                    "post_only without price"
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
                    Ok(OrderUpdate {
                        ts: decimal_to_ts(order.updated_at).ok_or_else(|| {
                            ExchangeError::Other(anyhow::anyhow!(
                                "parse ts error, ts={}",
                                order.updated_at
                            ))
                        })?,
                        order: ExcOrder {
                            id: OrderId::from(order.order_id),
                            target,
                            state,
                            trade: None,
                        },
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
