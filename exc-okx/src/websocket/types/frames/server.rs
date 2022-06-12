use crate::{
    error::OkxError,
    websocket::types::messages::event::{Change, TradeResponse},
};

use super::super::messages::event::{Event, ResponseKind};
use exc::types::ticker::Ticker;

/// Server Frame.
#[derive(Debug, Clone)]
pub struct ServerFrame {
    pub(crate) stream_id: usize,
    /// Inner Event.
    pub inner: Event,
}

impl ServerFrame {
    pub(crate) fn is_end_stream(&self) -> bool {
        matches!(
            self.inner,
            Event::Response(
                ResponseKind::Unsubscribe { arg: _ }
                    | ResponseKind::Error(_)
                    | ResponseKind::Login(_)
            )
        )
    }

    pub(crate) fn into_change(self) -> Option<Change> {
        match self.inner {
            Event::Change(change) => Some(change),
            _ => None,
        }
    }

    pub(crate) fn into_response(self) -> Option<ResponseKind> {
        match self.inner {
            Event::Response(resp) => Some(resp),
            _ => None,
        }
    }

    pub(crate) fn into_trade_response(self) -> Option<TradeResponse> {
        match self.inner {
            Event::TradeResponse(resp) => Some(resp),
            _ => None,
        }
    }
}

impl TryFrom<ServerFrame> for Vec<Result<Ticker, OkxError>> {
    type Error = OkxError;

    fn try_from(value: ServerFrame) -> Result<Self, Self::Error> {
        value.inner.try_into()
    }
}
