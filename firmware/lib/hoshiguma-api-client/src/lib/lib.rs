use hoshiguma_api::{Message, MessagePayload};
use log::info;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn send_command<
    Req: MessagePayload + Serialize + core::fmt::Debug,
    Resp: MessagePayload + DeserializeOwned + core::fmt::Debug,
>(
    stream: &mut TcpStream,
    request: Req,
) {
    let message = Message::new(&request).unwrap();
    let bytes = message.to_bytes().unwrap();
    stream.write_all(&bytes).await.unwrap();
    let request_time = std::time::Instant::now();

    let mut bytes = [0u8; 256];
    let n = stream.read(&mut bytes).await.unwrap();
    let bytes = &mut bytes[..n];
    let mut response_message = Message::from_bytes(bytes).unwrap();
    let response: Resp = response_message.payload().unwrap();
    let response_time = std::time::Instant::now();

    let duration = response_time.duration_since(request_time);
    info!("{request:?} => {response:?} in {}ms", duration.as_millis());
}
