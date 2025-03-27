use core::time::Duration;
use defmt::{trace, warn};
use embedded_io_async::{Read, Write};
use heapless::Vec;
use serde::{de::DeserializeOwned, Serialize};

pub struct EioTransport<T: Read + Write> {
    port: T,
}

impl<T: Read + Write> EioTransport<T> {
    pub fn new(port: T) -> Self {
        Self { port }
    }
}

impl<T: Read + Write, M: Serialize + DeserializeOwned> super::Transport<M> for EioTransport<T> {
    async fn flush(&mut self, timeout: Duration) -> Result<usize, crate::Error> {
        let mut count: usize = 0;

        loop {
            let mut b = [0u8];

            match embassy_time::with_timeout(
                embassy_time::Duration::from_micros(timeout.as_micros() as u64),
                self.port.read(&mut b),
            )
            .await
            {
                Ok(Ok(_)) => {
                    count = count.saturating_add(1);
                }
                Ok(Err(_)) => {
                    warn!("UART read fail");
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
        let mut buffer: Vec<u8, 512> = Vec::new();

        let start = embassy_time::Instant::now();

        loop {
            let mut b = [0u8];

            match embassy_time::with_timeout(
                embassy_time::Duration::from_millis(10),
                self.port.read(&mut b),
            )
            .await
            {
                Ok(Ok(_)) => {
                    buffer.extend(b);

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
                Ok(Err(_)) => {
                    warn!("UART read fail");
                }
                Err(_) => {
                    let elapsed = embassy_time::Instant::now() - start;
                    if elapsed.as_micros() as u128 >= timeout.as_micros() {
                        return Err(crate::Error::Timeout);
                    }
                }
            }
        }
    }

    async fn transmit_message(&mut self, msg: M) -> Result<(), crate::Error> {
        let mut buffer = [0u8; 512];

        let buffer =
            postcard::to_slice_cobs(&msg, &mut buffer).map_err(|_| crate::Error::SerializeError)?;

        self.port
            .write_all(buffer)
            .await
            .map_err(|_| crate::Error::TransportError)
    }
}
