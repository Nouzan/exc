use crate::{error::OkxError, websocket::types::messages::event::Change};

use super::super::messages::event::{Event, ResponseKind};
use exc_core::types::ticker::Ticker;
use serde::Deserialize;

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

    /// Get change if it is.
    pub fn change(&self) -> Option<&Change> {
        match &self.inner {
            Event::Change(change) => Some(change),
            _ => None,
        }
    }

    /// Get response if it is.
    pub fn response(&self) -> Option<&ResponseKind> {
        match &self.inner {
            Event::Response(resp) => Some(resp),
            _ => None,
        }
    }

    /// Convert to change.
    pub fn into_change(self) -> Option<Change> {
        match self.inner {
            Event::Change(change) => Some(change),
            _ => None,
        }
    }

    /// Convert to response.
    pub fn into_response(self) -> Option<ResponseKind> {
        match self.inner {
            Event::Response(resp) => Some(resp),
            _ => None,
        }
    }

    /// Deserialize change.
    pub fn into_deserialized_changes<T>(
        self,
    ) -> Option<impl Iterator<Item = Result<T, serde_json::Error>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        Some(self.into_change()?.deserialize_data())
    }

    /// Deserialize change.
    pub fn to_deserialized_changes<T>(
        &self,
    ) -> Option<impl Iterator<Item = Result<T, serde_json::Error>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        Some(self.change()?.clone().deserialize_data())
    }
}

impl TryFrom<ServerFrame> for Vec<Result<Ticker, OkxError>> {
    type Error = OkxError;

    fn try_from(value: ServerFrame) -> Result<Self, Self::Error> {
        value.inner.try_into()
    }
}
