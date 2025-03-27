use crate::{error, trace, warn};
use core::{marker::PhantomData, time::Duration};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct TokioChannelTransport<M> {
    tx: Sender<u8>,
    rx: Receiver<u8>,
    _msg_type: PhantomData<M>,
}

impl<M> TokioChannelTransport<M> {
    pub fn new_pair(capacity: usize) -> (Self, Self) {
        let (tx1, rx1) = channel::<u8>(capacity);
        let (tx2, rx2) = channel::<u8>(capacity);
        let transport_1 = Self {
            tx: tx1,
            rx: rx2,
            _msg_type: PhantomData,
        };
        let transport_2 = Self {
            tx: tx2,
            rx: rx1,
            _msg_type: PhantomData,
        };
        (transport_1, transport_2)
    }

    pub(crate) async fn transmit_raw(&mut self, data: &[u8]) -> Result<(), crate::Error> {
        for b in data {
            self.tx
                .send(*b)
                .await
                .map_err(|_| crate::Error::TransportError)?;
        }

        Ok(())
    }
}

impl<M: Serialize + DeserializeOwned> super::Transport<M> for TokioChannelTransport<M> {
    async fn flush(&mut self, timeout: Duration) -> Result<usize, crate::Error> {
        let mut count: usize = 0;

        loop {
            match tokio::time::timeout(timeout, self.rx.recv()).await {
                Ok(Some(_)) => {
                    count = count.saturating_add(1);
                }
                Ok(None) => {
                    warn!("Channel closed");
                    return Err(crate::Error::TransportError);
                }
                Err(_) => {
                    break;
                }
            }
        }

        Ok(count)
    }

    async fn receive_message(&mut self, timeout: Duration) -> Result<M, crate::Error> {
        let mut buffer = Vec::new();

        let start = tokio::time::Instant::now();

        loop {
            match tokio::time::timeout(Duration::from_millis(10), self.rx.recv()).await {
                Ok(Some(b)) => {
                    buffer.push(b);

                    if buffer.last() == Some(&0u8) {
                        match postcard::from_bytes_cobs::<M>(buffer.as_mut_slice()) {
                            Ok(msg) => {
                                trace!("Received message");
                                buffer.clear();
                                return Ok(msg);
                            }
                            Err(_) => {
                                warn!(
                                    "Failed to decode message with {} bytes in buffer",
                                    buffer.len(),
                                );
                                buffer.clear();
                                return Err(crate::Error::TransportError);
                            }
                        }
                    }
                }
                Ok(None) => {
                    warn!("Channel closed");
                    return Err(crate::Error::TransportError);
                }
                Err(_) => {
                    let elapsed = tokio::time::Instant::now() - start;
                    if elapsed.as_micros() >= timeout.as_micros() {
                        return Err(crate::Error::Timeout);
                    }
                }
            }
        }
    }

    async fn transmit_message(&mut self, msg: M) -> Result<(), crate::Error> {
        let buffer = postcard::to_stdvec_cobs(&msg).map_err(|e| {
            error!("Serialize error: {e}");
            crate::Error::SerializeError
        })?;

        self.transmit_raw(&buffer).await
    }
}
