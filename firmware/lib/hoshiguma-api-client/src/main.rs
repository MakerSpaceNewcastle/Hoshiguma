use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    let a = tokio::spawn(async {
        let request = hoshiguma_api::cooler::Request::SetCompressorState(
            hoshiguma_api::cooler::CompressorState::Run,
        );
        let mut stream = TcpStream::connect("10.69.69.4:1234").await.unwrap();
        let response: hoshiguma_api::cooler::Response =
            send_command(&mut stream, request.clone()).await;
        println!("Response: {:?}", response);
    });
    let b = tokio::spawn(async {
        let request = hoshiguma_api::cooler::Request::SetCompressorState(
            hoshiguma_api::cooler::CompressorState::Run,
        );
        let mut stream = TcpStream::connect("10.69.69.4:1234").await.unwrap();
        let response: hoshiguma_api::cooler::Response = send_command(&mut stream, request).await;
        println!("Response: {:?}", response);
    });
    let c = tokio::spawn(async {
        let request = hoshiguma_api::cooler::Request::SetCompressorState(
            hoshiguma_api::cooler::CompressorState::Run,
        );
        let mut stream = TcpStream::connect("10.69.69.4:1234").await.unwrap();
        let response: hoshiguma_api::cooler::Response = send_command(&mut stream, request).await;
        println!("Response: {:?}", response);
    });

    let _ = tokio::join!(a, b, c);
}

async fn send_command<Req: Serialize, Resp: DeserializeOwned>(
    stream: &mut TcpStream,
    command: Req,
) -> Resp {
    println!("Sending command");
    let bytes = postcard::to_stdvec_cobs(&command).unwrap();
    stream.write_all(&bytes).await.unwrap();
    let request_time = std::time::Instant::now();

    println!("Waiting for response...");
    let mut bytes = [0u8; 256];
    let n = stream.read(&mut bytes).await.unwrap();
    let bytes = &mut bytes[..n];
    let response = postcard::from_bytes_cobs(bytes).unwrap();
    let response_time = std::time::Instant::now();
    let duration = response_time.duration_since(request_time);
    println!("Got response in {}ms", duration.as_millis());

    response
}
