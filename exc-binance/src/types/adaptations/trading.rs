use std::collections::HashMap;

use exc_core::{types, Adaptor, ExchangeError};
use futures::{StreamExt, TryStreamExt};
use rust_decimal::Decimal;

use crate::{
    types::{
        trading::{OrderSide, Status, TimeInForce},
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
                let mut fees = HashMap::default();
                if let Some(asset) = update.fee_asset {
                    fees.insert(asset, -update.fee);
                }
                Ok(types::Order {
                    id: types::OrderId::from(update.client_id),
                    target: types::Place { size, kind },
                    state: types::OrderState {
                        filled,
                        cost: if filled.is_zero() {
                            Decimal::ONE
                        } else {
                            update.cost
                        },
                        base_fee: Decimal::ZERO,
                        quote_fee: Decimal::ZERO,
                        status,
                        fees,
                    },
                })
            })
            .boxed())
    }
}
