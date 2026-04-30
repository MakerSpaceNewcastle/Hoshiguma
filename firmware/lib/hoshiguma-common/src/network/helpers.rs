use super::Error;
use defmt::warn;
use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;
use hoshiguma_api::{CobsFramer, Message};

pub(super) async fn send_one<'a>(
    socket: &mut TcpSocket<'a>,
    message: &Message,
) -> Result<(), Error> {
    let response_bytes = message.to_bytes().map_err(|_| Error::MessageSerialize)?;

    socket.write_all(&response_bytes).await.map_err(|e| {
        warn!("{:?}", e);
        Error::SocketWrite
    })
}

pub(super) async fn receive_one<'a>(
    framer: &mut CobsFramer<4096>,
    socket: &mut TcpSocket<'a>,
) -> Result<Message, Error> {
    loop {
        let mut rx_buffer = [0; 1024];
        let bytes_received = match socket.read(&mut rx_buffer).await {
            Ok(0) => {
                return Err(Error::SocketReadEof);
            }
            Ok(n) => n,
            Err(e) => {
                warn!("{:?}", e);
                return Err(Error::SocketRead);
            }
        };

        framer
            .push(&rx_buffer[..bytes_received])
            .expect("should not be in the situation where the frame buffer is full");

        if let Some(mut message_data) = framer.next_message() {
            return Message::from_bytes(message_data.as_mut_slice())
                .map_err(|_| Error::MessageDeserialize);
        }
    }
}
