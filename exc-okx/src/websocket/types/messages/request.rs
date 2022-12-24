use super::Args;
use crate::error::OkxError;
use crate::key::{OkxKey as Key, Signature};
use exc_core::types::trading::{OrderKind, Place, PlaceOrderOptions};
use exc_core::types::TimeInForce;
use exc_core::Str;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use tokio_tungstenite::tungstenite::Message;

/// Okx websocket operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Op {
    /// Subscribe.
    Subscribe,
    /// Unsubsribe.
    Unsubscribe,
    /// Login.
    Login,
    /// Order.
    Order,
    /// Cancel order.
    CancelOrder,
}

/// Okx websocket request messagee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequestMessage {
    /// Id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Operation.
    pub op: Op,
    /// Arguments.
    #[serde(default)]
    pub args: Vec<Args>,
}

impl WsRequestMessage {
    /// Convert into a websocket message.
    pub fn to_websocket(&self) -> Result<Message, OkxError> {
        let text = serde_json::to_string(&self)?;
        Ok(Message::Text(text))
    }
}

/// Okx websocket request.
#[derive(Debug, Clone)]
pub enum WsRequest {
    /// Subscribe.
    Subscribe(Args),
    /// Unsubscribe.
    Unsubscribe(Args),
    /// Login.
    Login(Args),
    /// Order.
    Order(String, Args),
    /// Cancel order.
    CancelOrder(String, Args),
}

impl fmt::Display for WsRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subscribe(args) => {
                write!(f, "sub:{args}")
            }
            Self::Unsubscribe(args) => {
                write!(f, "unsub:{args}")
            }
            Self::Login(_args) => {
                write!(f, "login")
            }
            Self::Order(id, args) => {
                write!(f, "order:{id}:{args}")
            }
            Self::CancelOrder(id, args) => {
                write!(f, "cancel-order:{id}:{args}")
            }
        }
    }
}

impl WsRequest {
    /// Login request.
    pub(crate) fn login(key: Key, signature: Signature) -> Self {
        Self::Login(Args(BTreeMap::from([
            (Str::new_inline("apiKey"), key.apikey),
            (Str::new_inline("passphrase"), key.passphrase),
            (Str::new_inline("timestamp"), signature.timestamp),
            (Str::new_inline("sign"), signature.signature),
        ])))
    }

    /// Order request.
    pub(crate) fn order(place: &Place, opts: &PlaceOrderOptions) -> Self {
        let inst = opts.instrument();
        let custom = opts.custom();
        let size = place.size.abs();
        let side = if place.size.is_sign_negative() {
            "sell"
        } else {
            "buy"
        };
        let mut map = BTreeMap::from([
            (Str::new_inline("instId"), Str::new(inst)),
            (
                Str::new_inline("tdMode"),
                Str::new(custom.get("tdMode").map(|s| s.as_str()).unwrap_or("cross")),
            ),
            (Str::new_inline("side"), Str::new_inline(side)),
            (Str::new_inline("posSide"), Str::new_inline("net")),
            (Str::new_inline("sz"), Str::new(size.to_string())),
        ]);
        if let Some(margin) = opts.margin() {
            map.insert(Str::new_inline("ccy"), Str::new(margin));
        }
        #[cfg(not(feature = "prefer-client-id"))]
        if let Some(client_id) = opts.client_id() {
            #[cfg(debug_assertions)]
            if client_id.len() > 32 {
                tracing::error!(%client_id, "the length of `client_id` cannot be greater than 32");
            }
            map.insert(Str::new_inline("clOrdId"), Str::new(client_id));
        }
        #[cfg(feature = "prefer-client-id")]
        {
            let client_id = if let Some(client_id) = opts.client_id() {
                Str::new(client_id)
            } else {
                Str::new(uuid::Uuid::new_v4().simple().to_string())
            };
            #[cfg(debug_assertions)]
            if client_id.len() > 32 {
                tracing::error!(%client_id, "the length of `client_id` cannot be greater than 32");
            }
            map.insert(Str::new_inline("clOrdId"), client_id);
        }
        match place.kind {
            OrderKind::Market => {
                map.insert(Str::new_inline("ordType"), Str::new_inline("market"));
                if inst.split('-').count() == 2 {
                    // spot-market
                    map.insert(Str::new_inline("tgtCcy"), Str::new_inline("base_ccy"));
                }
            }
            OrderKind::Limit(price, tif) => {
                map.insert(Str::new_inline("px"), Str::new(price.to_string()));
                let t = match tif {
                    TimeInForce::GoodTilCancelled => "limit",
                    TimeInForce::FillOrKill => "fok",
                    TimeInForce::ImmediateOrCancel => "ioc",
                };
                map.insert(Str::new_inline("ordType"), Str::new_inline(t));
            }
            OrderKind::PostOnly(price) => {
                map.insert(Str::new_inline("px"), Str::new(price.to_string()));
                map.insert(Str::new_inline("ordType"), Str::new_inline("post_only"));
            }
        }
        Self::Order(format!("{:x}", uuid::Uuid::new_v4().as_u128()), Args(map))
    }

    /// Cancel order request.
    pub(crate) fn cancel_order(inst: &str, id: &str) -> Self {
        Self::CancelOrder(
            format!("{:x}", uuid::Uuid::new_v4().as_u128()),
            Args(BTreeMap::from([
                (Str::new_inline("instId"), Str::new(inst)),
                #[cfg(not(feature = "prefer-client-id"))]
                (Str::new_inline("ordId"), Str::new(id)),
                #[cfg(feature = "prefer-client-id")]
                (Str::new_inline("clOrdId"), Str::new(id)),
            ])),
        )
    }
}

impl From<WsRequest> for WsRequestMessage {
    fn from(req: WsRequest) -> Self {
        match req {
            WsRequest::Subscribe(args) => Self {
                id: None,
                op: Op::Subscribe,
                args: vec![args],
            },
            WsRequest::Unsubscribe(args) => Self {
                id: None,
                op: Op::Unsubscribe,
                args: vec![args],
            },
            WsRequest::Login(args) => Self {
                id: None,
                op: Op::Login,
                args: vec![args],
            },
            WsRequest::Order(id, args) => Self {
                id: Some(id),
                op: Op::Order,
                args: vec![args],
            },
            WsRequest::CancelOrder(id, args) => Self {
                id: Some(id),
                op: Op::CancelOrder,
                args: vec![args],
            },
        }
    }
}
