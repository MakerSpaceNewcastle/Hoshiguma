use hoshiguma_api::{CobsFramer, ExpectedResponse, Message, MessageError, MessagePayload};
use log::{debug, info, warn};
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::Instant,
};

#[derive(Debug)]
pub enum SendRequestError {
    Serialize(postcard::Error),
    Io(std::io::Error),
    PeerDisconnected,
    Deserialize(postcard::Error),
    Message(MessageError),
}

impl fmt::Display for SendRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(e) => write!(f, "failed to serialize request: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::PeerDisconnected => write!(f, "peer disconnected"),
            Self::Deserialize(e) => write!(f, "failed to deserialize response: {e}"),
            Self::Message(e) => write!(f, "message error: {e:?}"),
        }
    }
}

impl std::error::Error for SendRequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialize(e) | Self::Deserialize(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SendRequestError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<MessageError> for SendRequestError {
    fn from(e: MessageError) -> Self {
        Self::Message(e)
    }
}

pub async fn send_request<
    Req: ExpectedResponse<Response = Resp> + MessagePayload + Serialize + core::fmt::Debug,
    Resp: MessagePayload + DeserializeOwned + core::fmt::Debug,
>(
    stream: &mut TcpStream,
    request: Req,
) -> Result<Resp, SendRequestError> {
    // Serialize and send the request
    let message = Message::new(&request).map_err(SendRequestError::Serialize)?;
    let bytes = message.to_bytes().map_err(SendRequestError::Serialize)?;
    stream.write_all(&bytes).await?;
    let request_time = Instant::now();

    // Read and deserialize the response
    let mut bytes = receive_one(stream).await;
    if bytes.is_empty() {
        return Err(SendRequestError::PeerDisconnected);
    }
    let mut response_message =
        Message::from_bytes(&mut bytes).map_err(SendRequestError::Deserialize)?;
    let response: Resp = response_message.payload()?;
    let response_time = Instant::now();

    let duration = response_time.duration_since(request_time);
    info!("{request:?} => {response:?} in {}ms", duration.as_millis());

    if duration > Duration::from_millis(20) {
        warn!("Response took a long time!");
    }

    Ok(response)
}

pub async fn message_handler(stream: &mut TcpStream, f: impl Fn(Message) -> Message) {
    loop {
        let mut bytes = receive_one(stream).await;
        if bytes.is_empty() {
            debug!("Peer disconnected.");
            break;
        }
        let request_message = Message::from_bytes(&mut bytes).unwrap();

        let response_message = f(request_message);
        let response_bytes = response_message.to_bytes().unwrap();
        stream.write_all(&response_bytes).await.unwrap();
    }
}

async fn receive_one(stream: &mut TcpStream) -> Vec<u8> {
    let mut framer = CobsFramer::<4096>::default();
    loop {
        let mut bytes = [0u8; 256];
        let n = match stream.read(&mut bytes).await {
            Ok(0) => return Vec::new(), // Peer disconnected
            Ok(n) => n,
            Err(e) => {
                warn!("Failed to read from stream: {e:?}");
                return Vec::new();
            }
        };
        framer.push(&bytes[..n]).unwrap();

        if let Some(frame) = framer.next_message() {
            assert!(framer.is_empty());
            return frame.to_vec();
        }
    }
}
