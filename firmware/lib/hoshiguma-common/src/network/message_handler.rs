use defmt::{info, warn};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::Duration;
use embedded_io_async::Write;
use hoshiguma_api::{CONTROL_PORT, CobsFramer, Message};

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
        socket.set_timeout(Some(Duration::from_secs(5)));

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

        loop {
            let mut rx_buffer = [0; 1024];
            let bytes_received = match socket.read(&mut rx_buffer).await {
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

            framer
                .push(&rx_buffer[..bytes_received])
                .expect("should not be in the situation where the frame buffer is full");

            if let Some(mut message_data) = framer.next_message() {
                let message = match Message::from_bytes(message_data.as_mut_slice()) {
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
}
