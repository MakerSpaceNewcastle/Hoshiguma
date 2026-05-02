use super::{Error, receive_one, send_one, try_close, try_connect};
use defmt::{debug, warn};
use embassy_net::{Ipv4Address, Stack};
use embassy_time::Instant;
use hoshiguma_api::{CobsFramer, Message, MessagePayload};
use serde::{Serialize, de::DeserializeOwned};

pub async fn send_request<
    Request: MessagePayload + Serialize,
    Response: MessagePayload + DeserializeOwned,
>(
    stack: Stack<'static>,
    addr: Ipv4Address,
    port: u16,
    request: &Request,
) -> Result<Response, Error> {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let start = Instant::now();

    let mut socket = try_connect(stack, &mut rx_buffer, &mut tx_buffer, addr, port).await?;

    let tx_message = Message::new(request).map_err(|_| Error::MessageSerialize)?;
    send_one(&mut socket, &tx_message).await?;

    let mut framer = CobsFramer::<4096>::default();
    let rx_result = receive_one(&mut framer, &mut socket).await;

    try_close(&mut socket).await;
    drop(socket);

    if rx_result.is_ok() && !framer.is_empty() {
        warn!(
            "Framer buffer not empty after receiving single message: {} bytes left",
            framer.len()
        );
    }

    let result = rx_result?.payload().map_err(|_| Error::MessageDeserialize);

    let end = Instant::now();
    let duration = end - start;
    debug!("Request completed in {} ms", duration.as_millis());

    result
}
