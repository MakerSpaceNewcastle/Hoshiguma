use hoshiguma_api::{
    MessagePayload, bytes_to_payload,
    cooler::{CompressorState, Request, Response},
    payload_to_bytes,
};
use log::info;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    const ADDR: &str = "10.69.69.5:2001";

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect(ADDR).await.unwrap();
            send_command::<_, Response>(
                &mut stream,
                Request::SetCompressorState(CompressorState::Run),
            )
            .await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let mut stream = TcpStream::connect(ADDR).await.unwrap();
            send_command::<_, Response>(
                &mut stream,
                Request::SetCompressorState(CompressorState::Idle),
            )
            .await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let b = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect(ADDR).await.unwrap();
            send_command::<_, Response>(&mut stream, Request::GetGitRevision).await;
            send_command::<_, Response>(&mut stream, Request::GetBootReason).await;
            send_command::<_, Response>(&mut stream, Request::GetUptime).await;
            send_command::<_, Response>(&mut stream, Request::GetCompressorState).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let c = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect(ADDR).await.unwrap();
            send_command::<_, Response>(&mut stream, Request::GetTemperatures).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        }
    });

    let _ = tokio::join!(a, b, c);
}

async fn send_command<
    Req: MessagePayload + Serialize + core::fmt::Debug,
    Resp: MessagePayload + DeserializeOwned + core::fmt::Debug,
>(
    stream: &mut TcpStream,
    request: Req,
) {
    let bytes = payload_to_bytes(&request).unwrap();
    stream.write_all(&bytes).await.unwrap();
    let request_time = std::time::Instant::now();

    let mut bytes = [0u8; 256];
    let n = stream.read(&mut bytes).await.unwrap();
    let bytes = &mut bytes[..n];
    let response: Resp = bytes_to_payload(bytes).unwrap();
    let response_time = std::time::Instant::now();

    let duration = response_time.duration_since(request_time);
    info!("{request:?} => {response:?} in {}ms", duration.as_millis());
}
