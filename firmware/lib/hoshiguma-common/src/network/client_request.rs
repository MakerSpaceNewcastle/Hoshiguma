use super::{Error, receive_one, send_one};
use defmt::{debug, error, info, warn};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::Duration;
use hoshiguma_api::{CobsFramer, Message};

pub async fn send_request(
    stack: Stack<'static>,
    addr: embassy_net::Ipv4Address,
    port: u16,
    message: &Message,
) -> Result<Message, Error> {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(1)));

    'connect: for attempt in 1..=5 {
        debug!("Connecting to TCP {}:{} (attempt {})", addr, port, attempt);
        match socket.connect((addr, port)).await {
            Ok(_) => break 'connect,
            Err(e) => {
                warn!("Failed to connect to TCP {}:{}: {:?}", addr, port, e);
                continue 'connect;
            }
        };
    }

    if socket.remote_endpoint().is_none() {
        error!(
            "Failed to connect to TCP {}:{} after multiple attempts",
            addr, port
        );
        return Err(Error::NotConnected);
    }
    info!("Connected to TCP {}:{}", addr, port);

    send_one(&mut socket, message).await?;

    let mut framer = CobsFramer::<4096>::default();
    let res = receive_one(&mut framer, &mut socket).await;

    if res.is_ok() && !framer.is_empty() {
        warn!(
            "Framer buffer not empty after receiving single message: {} bytes left",
            framer.len()
        );
    }

    res
}
