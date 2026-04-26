use hoshiguma_api::{CobsFramer, Message, MessagePayload};
use log::info;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::Instant,
};

pub async fn send_command<
    Req: MessagePayload + Serialize + core::fmt::Debug,
    Resp: MessagePayload + DeserializeOwned + core::fmt::Debug,
>(
    stream: &mut TcpStream,
    request: Req,
) {
    // Serialize and send the request
    let message = Message::new(&request).unwrap();
    let bytes = message.to_bytes().unwrap();
    stream.write_all(&bytes).await.unwrap();
    let request_time = Instant::now();

    // Read and deserialize the response
    let mut bytes = receive_one(stream).await;
    let mut response_message = Message::from_bytes(&mut bytes).unwrap();
    let response: Resp = response_message.payload().unwrap();
    let response_time = Instant::now();

    let duration = response_time.duration_since(request_time);
    info!("{request:?} => {response:?} in {}ms", duration.as_millis());
}

async fn receive_one(stream: &mut TcpStream) -> Vec<u8> {
    let mut framer = CobsFramer::<4096>::default();
    loop {
        let mut bytes = [0u8; 256];
        let n = stream.read(&mut bytes).await.unwrap();
        framer.push(&bytes[..n]).unwrap();

        if let Some(frame) = framer.next_message() {
            assert!(framer.is_empty());
            return frame.to_vec();
        }
    }
}
