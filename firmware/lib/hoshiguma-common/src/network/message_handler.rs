use defmt::{Format, info, warn};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::Duration;
use embedded_io_async::Write;
use hoshiguma_api::{CONTROL_PORT, CobsFramer, Message};

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    SocketRead,
    SocketReadEof,
    SocketWrite,
    MessageDeserialize,
    MessageSerialize,
}

pub async fn message_handler_loop<F: AsyncFnMut(Message) -> Message>(
    stack: Stack<'static>,
    id: u8,
    mut handler: F,
) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut framer = CobsFramer::<4096>::default();

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(1)));

        info!("socket {}: Listening on TCP:{}...", id, CONTROL_PORT);
        if let Err(e) = socket.accept(CONTROL_PORT).await {
            warn!("socket {}: accept error: {:?}", id, e);
            continue;
        }
        info!(
            "socket {}: connection from {:?}",
            id,
            socket.remote_endpoint()
        );

        let message = match receive_one(&mut framer, &mut socket).await {
            Ok(message) => message,
            Err(e) => {
                warn!("socket {}: failed to receive message: {}", id, e);
                continue;
            }
        };

        let message = handler(message).await;

        if let Err(e) = send_one(&mut socket, &message).await {
            warn!("socket {}: failed to send response: {}", id, e);
        };
    }
}

// https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/ethernet_w5500_tcp_client.rs
pub async fn send_request<'a>(
    socket: &mut TcpSocket<'a>,
    message: &Message,
) -> Result<Message, Error> {
    send_one(socket, message).await?;

    let mut framer = CobsFramer::<4096>::default();
    let res = receive_one(&mut framer, socket).await;

    if res.is_ok() && !framer.is_empty() {
        warn!(
            "framer buffer not empty after receiving single message: {} bytes left",
            framer.len()
        );
    }

    res
}

async fn send_one<'a>(socket: &mut TcpSocket<'a>, message: &Message) -> Result<(), Error> {
    let response_bytes = message.to_bytes().map_err(|_| Error::MessageSerialize)?;

    socket.write_all(&response_bytes).await.map_err(|e| {
        warn!("{:?}", e);
        Error::SocketWrite
    })
}

async fn receive_one<'a>(
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
