use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
};

use tokio::sync::broadcast;

use crate::websocket::protocol::frame::{Name, StreamFrame};

const CAP: usize = 64;

pub(super) struct MainStream {
    pub_sub: Mutex<HashMap<Name, broadcast::Sender<StreamFrame>>>,
}

impl MainStream {
    pub(super) fn new<T>(names: T) -> Self
    where
        T: IntoIterator<Item = Name>,
    {
        let pub_sub = names
            .into_iter()
            .map(|name| (name, broadcast::channel(CAP).0))
            .collect();
        Self {
            pub_sub: Mutex::new(pub_sub),
        }
    }
}

impl MainStream {
    pub(super) fn try_publish(
        &self,
        name: &Name,
        frame: StreamFrame,
    ) -> Result<usize, StreamFrame> {
        if let Some(sender) = self.pub_sub.lock().unwrap().get(name) {
            tracing::trace!("received a main stream frame: {frame:?}");
            Ok(sender.send(frame).unwrap_or(0))
        } else {
            Err(frame)
        }
    }

    pub(super) fn subscribe(&self, name: &Name) -> Option<broadcast::Receiver<StreamFrame>> {
        match self.pub_sub.lock().unwrap().entry(name.clone()) {
            Entry::Occupied(e) => Some(e.get().subscribe()),
            Entry::Vacant(e) => {
                let (tx, rx) = broadcast::channel(CAP);
                e.insert(tx);
                Some(rx)
            }
        }
    }
}
