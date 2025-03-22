use core::time::Duration;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct TokioChannelTransport<M> {
    tx: Sender<M>,
    rx: Receiver<M>,
}

impl<M> TokioChannelTransport<M> {
    pub fn new_pair(capacity: usize) -> (Self, Self) {
        let (tx1, rx1) = channel::<M>(capacity);
        let (tx2, rx2) = channel::<M>(capacity);
        let transport_1 = Self { tx: tx1, rx: rx2 };
        let transport_2 = Self { tx: tx2, rx: rx1 };
        (transport_1, transport_2)
    }
}

impl<M: Serialize + DeserializeOwned> super::Transport<M> for TokioChannelTransport<M> {
    async fn receive_message(&mut self, timeout: Duration) -> Result<M, crate::Error> {
        match tokio::time::timeout(timeout, self.rx.recv()).await {
            Ok(Some(msg)) => Ok(msg),
            Ok(None) => Err(crate::Error::TransportError),
            Err(_) => Err(crate::Error::Timeout),
        }
    }

    async fn transmit_message(&mut self, msg: M) -> Result<(), crate::Error> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| crate::Error::TransportError)
    }
}
