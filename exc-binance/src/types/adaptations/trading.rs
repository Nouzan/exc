use std::collections::HashMap;

use exc_core::{types, Adaptor, ExchangeError};
use futures::{FutureExt, StreamExt, TryStreamExt};
use rust_decimal::Decimal;

use crate::{
    http::{
        request::trading::{CancelOrder, GetOrder, PlaceOrder},
        response::trading::Order,
    },
    types::{
        trading::{self, OrderSide, PositionSide, Status, TimeInForce},
        Name,
    },
    websocket::protocol::frame::account::{OrderType, OrderUpdate},
    Request,
};

impl Adaptor<types::SubscribeOrders> for Request {
    fn from_request(req: types::SubscribeOrders) -> Result<Self, ExchangeError> {
        Ok(Self::subscribe(Name::order_trade_update(&req.instrument)))
    }

    fn into_response(resp: Self::Response) -> Result<types::OrderStream, ExchangeError> {
        let stream = resp.into_stream::<OrderUpdate>()?;
        Ok(stream
            .map_err(ExchangeError::from)
            .and_then(|update| async move {
                let kind = match update.order_type {
                    OrderType::Limit => match update.time_in_force {
                        TimeInForce::Gtc => types::OrderKind::Limit(
                            update.price,
                            types::TimeInForce::GoodTilCancelled,
                        ),
                        TimeInForce::Fok => {
                            types::OrderKind::Limit(update.price, types::TimeInForce::FillOrKill)
                        }
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
            })
            .boxed())
    }
}

impl TryFrom<Order> for types::Order {
    type Error = ExchangeError;

    fn try_from(order: Order) -> Result<Self, Self::Error> {
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
                TimeInForce::Gtc => {
                    types::OrderKind::Limit(order.price, types::TimeInForce::GoodTilCancelled)
                }
                TimeInForce::Fok => {
                    types::OrderKind::Limit(order.price, types::TimeInForce::FillOrKill)
                }
                TimeInForce::Ioc => {
                    types::OrderKind::Limit(order.price, types::TimeInForce::ImmediateOrCancel)
                }
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
            Status::Canceled | Status::Expired | Status::Filled => types::OrderStatus::Finished,
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
}

impl Adaptor<types::PlaceOrder> for Request {
    fn from_request(req: types::PlaceOrder) -> Result<Self, ExchangeError> {
        let place = req.place;
        let side = if place.size.is_zero() {
            return Err(ExchangeError::Other(anyhow!("place zero size order")));
        } else if place.size.is_sign_positive() {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };
        let (order_type, price, tif) = match place.kind {
            types::OrderKind::Market => (trading::OrderType::Market, None, None),
            types::OrderKind::Limit(price, tif) => {
                let tif = match tif {
                    types::TimeInForce::GoodTilCancelled => Some(TimeInForce::Gtc),
                    types::TimeInForce::FillOrKill => Some(TimeInForce::Fok),
                    types::TimeInForce::ImmediateOrCancel => Some(TimeInForce::Ioc),
                };
                (trading::OrderType::Limit, Some(price), tif)
            }
            types::OrderKind::PostOnly(price) => (
                trading::OrderType::Limit,
                Some(price),
                Some(TimeInForce::Gtx),
            ),
        };
        Ok(Self::with_rest_payload(PlaceOrder {
            symbol: req.instrument.to_uppercase(),
            side,
            position_side: PositionSide::Both,
            order_type,
            reduce_only: None,
            quantity: Some(place.size.abs()),
            price,
            new_client_order_id: req.client_id,
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            time_in_force: tif,
            working_type: None,
            price_protect: None,
            new_order_resp_type: None,
        }))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<types::PlaceOrder as exc_core::Request>::Response, ExchangeError> {
        Ok(async move {
            let order = resp.into_response::<Order>()?;
            let id = types::OrderId::from(order.client_order_id.clone());
            Ok(types::Placed {
                ts: super::from_timestamp(order.update_time)?,
                id,
                order: Some(order.try_into()?),
            })
        }
        .boxed())
    }
}

impl Adaptor<types::CancelOrder> for Request {
    fn from_request(req: types::CancelOrder) -> Result<Self, ExchangeError> {
        Ok(Self::with_rest_payload(CancelOrder {
            symbol: req.instrument.to_uppercase(),
            order_id: None,
            orig_client_order_id: Some(req.id.as_str().to_string()),
        }))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<types::CancelOrder as exc_core::Request>::Response, ExchangeError> {
        Ok(async move {
            let order = resp.into_response::<Order>()?;
            Ok(types::Cancelled {
                ts: super::from_timestamp(order.update_time)?,
                order: Some(order.try_into()?),
            })
        }
        .boxed())
    }
}

impl Adaptor<types::GetOrder> for Request {
    fn from_request(req: types::GetOrder) -> Result<Self, ExchangeError> {
        Ok(Self::with_rest_payload(GetOrder {
            symbol: req.instrument.to_uppercase(),
            order_id: None,
            orig_client_order_id: Some(req.id.as_str().to_string()),
        }))
    }

    fn into_response(
        resp: Self::Response,
    ) -> Result<<types::GetOrder as exc_core::Request>::Response, ExchangeError> {
        Ok(async move {
            let order = resp.into_response::<Order>()?;
            Ok(types::OrderUpdate {
                ts: super::from_timestamp(order.update_time)?,
                order: order.try_into()?,
            })
        }
        .boxed())
    }
}
