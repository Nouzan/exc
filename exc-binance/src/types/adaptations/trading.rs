use std::collections::HashMap;

use either::Either;
use exc_core::{types, Adaptor, ExchangeError};
use futures::{FutureExt, StreamExt, TryStreamExt};
use rust_decimal::Decimal;
use time::OffsetDateTime;

use crate::{
    http::{
        request::trading::{CancelOrder, GetOrder, GetOrderInner, PlaceOrder},
        response::trading::Order,
    },
    types::{
        trading::{self, OrderSide, Status, TimeInForce},
        Name,
    },
    websocket::protocol::frame::account::{ExecutionReport, OrderType, OrderUpdate},
    Request,
};

type OrderUpdateKind = Either<OrderUpdate, ExecutionReport>;

impl Adaptor<types::SubscribeOrders> for Request {
    fn from_request(req: types::SubscribeOrders) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe(Name::order_trade_update(&req.instrument)))
    }

    fn into_response(resp: Self::Response) -> Result<types::OrderStream, ExchangeError> {
        let stream = resp.into_stream::<OrderUpdateKind>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|update| async move {
                match update {
                    Either::Left(update) => {
                        let kind = match update.order_type {
                            OrderType::Limit => match update.time_in_force {
                                TimeInForce::Gtc => types::OrderKind::Limit(
                                    update.price,
                                    types::TimeInForce::GoodTilCancelled,
                                ),
                                TimeInForce::Fok => types::OrderKind::Limit(
                                    update.price,
                                    types::TimeInForce::FillOrKill,
                                ),
                                TimeInForce::Ioc => types::OrderKind::Limit(
                                    update.price,
                                    types::TimeInForce::ImmediateOrCancel,
                                ),
                                TimeInForce::Gtx => types::OrderKind::PostOnly(update.price),
                            },
                            OrderType::Market => types::OrderKind::Market,
                            other => {
                                return Err(ExchangeError::Other(anyhow!(
                                    "unsupported order type: {other:?}"
                                )));
                            }
                        };
                        let mut filled = update.filled_size.abs();
                        let mut size = update.size.abs();
                        match update.side {
                            OrderSide::Buy => {
                                filled.set_sign_positive(true);
                                size.set_sign_positive(true)
                            }
                            OrderSide::Sell => {
                                filled.set_sign_positive(false);
                                size.set_sign_positive(false)
                            }
                        }
                        let status = match update.status {
                            Status::New | Status::PartiallyFilled => types::OrderStatus::Pending,
                            Status::Canceled | Status::Expired | Status::Filled => {
                                types::OrderStatus::Finished
                            }
                            Status::NewAdl | Status::NewInsurance => types::OrderStatus::Pending,
                        };
                        let trade_size = update.last_trade_size.abs();
                        let trade = if !trade_size.is_zero() {
                            let mut trade = types::OrderTrade {
                                price: update.last_trade_price,
                                size: if matches!(update.side, OrderSide::Buy) {
                                    trade_size
                                } else {
                                    -trade_size
                                },
                                fee: Decimal::ZERO,
                                fee_asset: None,
                            };
                            if let Some(asset) = update.fee_asset {
                                trade.fee = update.fee;
                                trade.fee_asset = Some(asset);
                            }
                            Some(trade)
                        } else {
                            None
                        };
                        Ok(types::OrderUpdate {
                            ts: super::from_timestamp(update.trade_ts)?,
                            order: types::Order {
                                id: types::OrderId::from(update.client_id),
                                target: types::Place { size, kind },
                                state: types::OrderState {
                                    filled,
                                    cost: if filled.is_zero() {
                                        Decimal::ONE
                                    } else {
                                        update.cost
                                    },
                                    status,
                                    fees: HashMap::default(),
                                },
                                trade,
                            },
                        })
                    }
                    Either::Right(update) => {
                        let kind = match update.order_type {
                            OrderType::Limit => match update.time_in_force {
                                TimeInForce::Gtc => types::OrderKind::Limit(
                                    update.price,
                                    types::TimeInForce::GoodTilCancelled,
                                ),
                                TimeInForce::Fok => types::OrderKind::Limit(
                                    update.price,
                                    types::TimeInForce::FillOrKill,
                                ),
                                TimeInForce::Ioc => types::OrderKind::Limit(
                                    update.price,
                                    types::TimeInForce::ImmediateOrCancel,
                                ),
                                TimeInForce::Gtx => types::OrderKind::PostOnly(update.price),
                            },
                            OrderType::Market => types::OrderKind::Market,
                            other => {
                                return Err(ExchangeError::Other(anyhow!(
                                    "unsupported order type: {other:?}"
                                )));
                            }
                        };
                        let mut filled = update.filled_size.abs();
                        let mut size = update.size.abs();
                        match update.side {
                            OrderSide::Buy => {
                                filled.set_sign_positive(true);
                                size.set_sign_positive(true)
                            }
                            OrderSide::Sell => {
                                filled.set_sign_positive(false);
                                size.set_sign_positive(false)
                            }
                        }
                        let status = match update.status {
                            Status::New | Status::PartiallyFilled => types::OrderStatus::Pending,
                            Status::Canceled | Status::Expired | Status::Filled => {
                                types::OrderStatus::Finished
                            }
                            Status::NewAdl | Status::NewInsurance => types::OrderStatus::Pending,
                        };
                        let trade_size = update.last_trade_size.abs();
                        let trade = if !trade_size.is_zero() {
                            let mut trade = types::OrderTrade {
                                price: update.last_trade_price,
                                size: if matches!(update.side, OrderSide::Buy) {
                                    trade_size
                                } else {
                                    -trade_size
                                },
                                fee: Decimal::ZERO,
                                fee_asset: None,
                            };
                            if let Some(asset) = update.fee_asset {
                                trade.fee = update.fee;
                                trade.fee_asset = Some(asset);
                            }
                            Some(trade)
                        } else {
                            None
                        };
                        Ok(types::OrderUpdate {
                            ts: super::from_timestamp(update.trade_ts)?,
                            order: types::Order {
                                id: types::OrderId::from(update.client_id),
                                target: types::Place { size, kind },
                                state: types::OrderState {
                                    filled,
                                    cost: if update.filled_size.is_zero() {
                                        Decimal::ONE
                                    } else {
                                        update.filled_quote_size / update.filled_size
                                    },
                                    status,
                                    fees: HashMap::default(),
                                },
                                trade,
                            },
                        })
                    }
                }
            })
            .boxed())
    }
}

impl TryFrom<Order> for types::Order {
    type Error = ExchangeError;

    fn try_from(order: Order) -> Result<Self, Self::Error> {
        match order {
            Order::UsdMarginFutures(order) => {
                let mut filled = order.executed_qty.abs();
                let mut size = order.orig_qty.abs();
                match order.side {
                    OrderSide::Buy => {
                        filled.set_sign_positive(true);
                        size.set_sign_positive(true);
                    }
                    OrderSide::Sell => {
                        filled.set_sign_positive(false);
                        size.set_sign_positive(false);
                    }
                }
                let kind = match order.order_type {
                    trading::OrderType::Limit => match order.time_in_force {
                        TimeInForce::Gtc => types::OrderKind::Limit(
                            order.price,
                            types::TimeInForce::GoodTilCancelled,
                        ),
                        TimeInForce::Fok => {
                            types::OrderKind::Limit(order.price, types::TimeInForce::FillOrKill)
                        }
                        TimeInForce::Ioc => types::OrderKind::Limit(
                            order.price,
                            types::TimeInForce::ImmediateOrCancel,
                        ),
                        TimeInForce::Gtx => types::OrderKind::PostOnly(order.price),
                    },
                    trading::OrderType::Market => types::OrderKind::Market,
                    other => {
                        return Err(ExchangeError::Other(anyhow!(
                            "unsupported order type: {other:?}"
                        )));
                    }
                };
                let status = match order.status {
                    Status::New | Status::PartiallyFilled => types::OrderStatus::Pending,
                    Status::Canceled | Status::Expired | Status::Filled => {
                        types::OrderStatus::Finished
                    }
                    Status::NewAdl | Status::NewInsurance => types::OrderStatus::Pending,
                };
                Ok(types::Order {
                    id: types::OrderId::from(order.client_order_id),
                    target: types::Place { size, kind },
                    state: types::OrderState {
                        filled,
                        cost: if filled.is_zero() {
                            Decimal::ONE
                        } else {
                            order.avg_price
                        },
                        status,
                        fees: HashMap::default(),
                    },
                    trade: None,
                })
            }
            Order::Spot(order) => {
                let ack = order.ack;
                if let Some(result) = order.result {
                    let mut filled = result.executed_qty.abs();
                    let mut size = result.orig_qty.abs();
                    match result.side {
                        OrderSide::Buy => {
                            filled.set_sign_positive(true);
                            size.set_sign_positive(true);
                        }
                        OrderSide::Sell => {
                            filled.set_sign_positive(false);
                            size.set_sign_positive(false);
                        }
                    }
                    let kind = match result.order_type {
                        trading::OrderType::Limit => match result.time_in_force {
                            TimeInForce::Gtc | TimeInForce::Gtx => types::OrderKind::Limit(
                                result.price,
                                types::TimeInForce::GoodTilCancelled,
                            ),
                            TimeInForce::Fok => types::OrderKind::Limit(
                                result.price,
                                types::TimeInForce::FillOrKill,
                            ),
                            TimeInForce::Ioc => types::OrderKind::Limit(
                                result.price,
                                types::TimeInForce::ImmediateOrCancel,
                            ),
                        },
                        trading::OrderType::Market => types::OrderKind::Market,
                        trading::OrderType::LimitMaker => types::OrderKind::PostOnly(result.price),
                        other => {
                            return Err(ExchangeError::Other(anyhow!(
                                "unsupported order type: {other:?}"
                            )));
                        }
                    };
                    let status = match result.status {
                        Status::New | Status::PartiallyFilled => types::OrderStatus::Pending,
                        Status::Canceled | Status::Expired | Status::Filled => {
                            types::OrderStatus::Finished
                        }
                        Status::NewAdl | Status::NewInsurance => types::OrderStatus::Pending,
                    };
                    let mut fees = HashMap::default();
                    let mut last_trade = None;
                    for fill in order.fills {
                        let fee = fees.entry(fill.commission_asset.clone()).or_default();
                        *fee -= fill.commission;
                        last_trade = Some(types::OrderTrade {
                            price: fill.price,
                            size: fill.qty,
                            fee: -fill.commission,
                            fee_asset: Some(fill.commission_asset),
                        });
                    }
                    Ok(types::Order {
                        id: types::OrderId::from(ack.client_order_id),
                        target: types::Place { size, kind },
                        state: types::OrderState {
                            filled,
                            cost: if result.executed_qty.is_zero() {
                                Decimal::ONE
                            } else {
                                result.cummulative_quote_qty / result.executed_qty
                            },
                            status,
                            fees,
                        },
                        trade: last_trade,
                    })
                } else {
                    Err(ExchangeError::Other(anyhow::anyhow!(
                        "order result is missing"
                    )))
                }
            }
        }
    }
}

impl Adaptor<types::PlaceOrder> for Request {
    fn from_request(req: types::PlaceOrder) -> Result<Self, ExchangeError> {
        Ok(Self::with_rest_payload(PlaceOrder { inner: req }))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<types::PlaceOrder as exc_core::Request>::Response, ExchangeError> {
        Ok(async move {
            let order = resp.into_response::<Order>()?;
            let id = types::OrderId::from(order.client_id().to_string());
            Ok(types::Placed {
                ts: order
                    .updated()
                    .map(super::from_timestamp)
                    .unwrap_or_else(|| Ok(OffsetDateTime::now_utc()))?,
                id,
                order: order.try_into().ok(),
            })
        }
        .boxed())
    }
}

impl Adaptor<types::CancelOrder> for Request {
    fn from_request(req: types::CancelOrder) -> Result<Self, ExchangeError> {
        Ok(Self::with_rest_payload(CancelOrder {
            inner: GetOrderInner {
                symbol: req.instrument.to_uppercase(),
                order_id: None,
                orig_client_order_id: Some(req.id.as_str().to_string()),
            },
        }))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<types::CancelOrder as exc_core::Request>::Response, ExchangeError> {
        Ok(async move {
            let order = resp.into_response::<Order>()?;
            Ok(types::Cancelled {
                ts: order
                    .updated()
                    .map(super::from_timestamp)
                    .unwrap_or_else(|| Ok(OffsetDateTime::now_utc()))?,
                order: Some(order.try_into()?),
            })
        }
        .boxed())
    }
}

impl Adaptor<types::GetOrder> for Request {
    fn from_request(req: types::GetOrder) -> Result<Self, ExchangeError> {
        Ok(Self::with_rest_payload(GetOrder {
            inner: GetOrderInner {
                symbol: req.instrument.to_uppercase(),
                order_id: None,
                orig_client_order_id: Some(req.id.as_str().to_string()),
            },
        }))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<types::GetOrder as exc_core::Request>::Response, ExchangeError> {
        Ok(async move {
            let order = resp.into_response::<Order>()?;
            Ok(types::OrderUpdate {
                ts: order
                    .updated()
                    .map(super::from_timestamp)
                    .unwrap_or_else(|| Ok(OffsetDateTime::now_utc()))?,
                order: order.try_into()?,
            })
        }
        .boxed())
    }
}
