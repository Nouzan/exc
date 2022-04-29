use tokio::sync::oneshot::{channel, Receiver, Sender};

pub(crate) struct Callback {
    pub(crate) tx: Option<Sender<()>>,
}

impl Callback {
    pub(crate) fn new() -> (Self, Receiver<()>) {
        let (tx, rx) = channel();
        (Self { tx: Some(tx) }, rx)
    }
}

impl Drop for Callback {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        }
    }
}
