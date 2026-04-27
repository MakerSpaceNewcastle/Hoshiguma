use crate::network::config::NOTIFICATION_PORT;
use core::net::Ipv4Addr;
use defmt::{info, warn};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use embedded_io_async::Write;
use hoshiguma_api::{Message, MessagePayload};
use serde::Serialize;

pub async fn notification_tx_loop<T: MessagePayload + Serialize, const CAP: usize>(
    stack: Stack<'static>,
    endpoints: &'static [Ipv4Addr],
    id: u8,
    notification_rx: Receiver<'static, CriticalSectionRawMutex, T, CAP>,
) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        let message = notification_rx.receive().await;

        if let Ok(message) = Message::new(&message) {
            'recip: for ip in endpoints.iter() {
                info!("socket {}: sending notification to {:?}", id, ip);

                let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
                if let Err(e) = socket.connect((*ip, NOTIFICATION_PORT)).await {
                    warn!("socket {}: failed to connect to {:?}: {:?}", id, ip, e);
                    continue 'recip;
                }

                let message_bytes = match message.to_bytes() {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        warn!("socket {}: failed to serialize message", id);
                        continue 'recip;
                    }
                };

                if let Err(e) = socket.write_all(&message_bytes).await {
                    warn!(
                        "socket {}: failed to send notification to {:?}: {:?}",
                        id, ip, e
                    );
                }
            }
        } else {
            warn!("socket {}: failed to create message", id);
        }
    }
}
