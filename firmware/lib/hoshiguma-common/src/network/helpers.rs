use defmt::{debug, info, warn};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::Duration;
use embedded_io_async::Write;
use hoshiguma_api::Message;

pub async fn message_handler_loop<F: AsyncFnMut(Message) -> Message>(
    stack: Stack<'static>,
    port: u16,
    id: u8,
    mut handler: F,
) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(5)));

        info!("socket {}: Listening on TCP:{}...", id, port);
        if let Err(e) = socket.accept(port).await {
            warn!("socket {}: accept error: {:?}", id, e);
            continue;
        }
        info!(
            "socket {}: connection from {:?}",
            id,
            socket.remote_endpoint()
        );

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    info!("socket {}: EOF", id);
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("socket {}: {:?}", id, e);
                    break;
                }
            };

            let received = &mut buf[..n];
            debug!("socket {}: received {} bytes", id, received.len());

            let message = match Message::from_bytes(received) {
                Ok(message) => message,
                Err(_) => {
                    warn!("socket {}: failed to parse message", id);
                    break;
                }
            };

            let message = handler(message).await;

            let response_bytes = match message.to_bytes() {
                Ok(message) => message,
                Err(_) => {
                    warn!("socket {}: failed to serialize response message", id);
                    continue;
                }
            };

            if let Err(e) = socket.write_all(&response_bytes).await {
                warn!("socket {}: write error: {:?}", id, e);
                break;
            }
        }
    }
}
