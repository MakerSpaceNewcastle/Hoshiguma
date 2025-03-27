use crate::error;
use core::time::Duration;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct SerialTransport {
    reader: BufReader<ReadHalf<SerialStream>>,
    writer: WriteHalf<SerialStream>,
}

impl SerialTransport {
    pub fn new(port: &str, baud: u32) -> Result<Self, crate::Error> {
        let port = tokio_serial::new(port, baud)
            .open_native_async()
            .map_err(|e| {
                error!("Failed to open serial port {port} with error {e}");
                crate::Error::TransportError
            })?;

        let (reader, writer) = tokio::io::split(port);
        let reader = BufReader::new(reader);

        Ok(Self { reader, writer })
    }
}

impl<M: Serialize + DeserializeOwned> super::Transport<M> for SerialTransport {
    async fn flush(&mut self, timeout: Duration) -> Result<usize, crate::Error> {
        let mut count: usize = 0;

        loop {
            match tokio::time::timeout(timeout, self.reader.read_u8()).await {
                Ok(Ok(_)) => {
                    count = count.saturating_add(1);
                }
                Ok(Err(_)) => {
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

        // Receive data
        let _num_bytes =
            match tokio::time::timeout(timeout, self.reader.read_until(b'\0', &mut buffer)).await {
                Ok(Ok(0)) => {
                    error!("No data received");
                    Err(crate::Error::TransportError)
                }
                Ok(Ok(num_bytes)) => Ok(num_bytes),
                Ok(Err(e)) => {
                    error!("Serial error: {e}");
                    Err(crate::Error::TransportError)
                }
                Err(_) => Err(crate::Error::Timeout),
            }?;

        // Decode message
        postcard::from_bytes_cobs::<M>(&mut buffer).map_err(|e| {
            error!("Deserialize error: {e}");
            crate::Error::DeserializeError
        })
    }

    async fn transmit_message(&mut self, msg: M) -> Result<(), crate::Error> {
        let buffer = postcard::to_stdvec_cobs(&msg).map_err(|e| {
            error!("Serialize error: {e}");
            crate::Error::SerializeError
        })?;

        self.writer.write_all(&buffer).await.map_err(|e| {
            error!("Failed to write to serial port with error {e}");
            crate::Error::TransportError
        })
    }
}
