use super::{Error, receive_one, send_one, try_close};
use defmt::{debug, info, warn};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::{Duration, Timer};
use hoshiguma_api::{CONTROL_PORT, CobsFramer, Message};

pub async fn message_handler_loop<F: AsyncFnMut(Message) -> Message>(
    stack: Stack<'static>,
    id: usize,
    mut handler: F,
) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut framer = CobsFramer::<4096>::default();

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(1)));

    'conn: loop {
        debug!("socket {}: listening on TCP {}...", id, CONTROL_PORT);
        if let Err(e) = socket.accept(CONTROL_PORT).await {
            warn!(
                "socket {}: accept error: {} (state {})",
                id,
                e,
                socket.state()
            );
            Timer::after(Duration::from_millis(500)).await;
            continue;
        }
        info!(
            "socket {}: connection from {}",
            id,
            socket.remote_endpoint()
        );

        loop {
            let message = match receive_one(&mut framer, &mut socket).await {
                Ok(message) => message,
                Err(Error::SocketReadEof) | Err(Error::ConnectionReset) => {
                    info!("socket {}: connection closed by peer", id);
                    try_close(&mut socket).await;
                    continue 'conn;
                }
                Err(e) => {
                    warn!("socket {}: failed to receive message: {}", id, e);
                    try_close(&mut socket).await;
                    continue 'conn;
                }
            };

            let message = handler(message).await;

            if let Err(e) = send_one(&mut socket, &message).await {
                warn!("socket {}: failed to send response: {}", id, e);
                try_close(&mut socket).await;
                continue 'conn;
            };
        }
    }
}
