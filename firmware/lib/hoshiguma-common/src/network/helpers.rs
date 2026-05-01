use super::Error;
use defmt::{debug, error, info, warn};
use embassy_net::{Ipv4Address, Stack, tcp::TcpSocket};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use hoshiguma_api::{CobsFramer, Message};

pub async fn try_connect<'a>(
    stack: Stack<'static>,
    rx_buffer: &'a mut [u8],
    tx_buffer: &'a mut [u8],
    addr: Ipv4Address,
    port: u16,
) -> Result<TcpSocket<'a>, Error> {
    let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(1)));

    'connect: for attempt in 1..=50 {
        debug!("Connecting to TCP {}:{} (attempt {})", addr, port, attempt);
        match socket.connect((addr, port)).await {
            Ok(_) => break 'connect,
            Err(e) => {
                warn!("Failed to connect to TCP {}:{}: {}", addr, port, e);
                Timer::after_millis(10).await;
                continue 'connect;
            }
        };
    }

    if socket.remote_endpoint().is_none() {
        error!(
            "Failed to connect to TCP {}:{} after multiple attempts",
            addr, port
        );
        Err(Error::NotConnected)
    } else {
        info!("Connected to TCP {}:{}", addr, port);
        Ok(socket)
    }
}

pub(super) async fn try_close<'a>(socket: &mut TcpSocket<'a>) {
    socket.close();
    if let Err(e) = socket.flush().await {
        warn!("Failed to flush socket after closing: {}", e);
    }
}

pub(super) async fn send_one<'a>(
    socket: &mut TcpSocket<'a>,
    message: &Message,
) -> Result<(), Error> {
    let response_bytes = message.to_bytes().map_err(|_| Error::MessageSerialize)?;

    socket.write_all(&response_bytes).await.map_err(|e| {
        warn!("{}", e);
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
            Err(embassy_net::tcp::Error::ConnectionReset) => {
                return Err(Error::ConnectionReset);
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
