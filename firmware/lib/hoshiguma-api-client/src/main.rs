use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

fn main() {
    let mut stream = TcpStream::connect("10.69.69.4:1234").unwrap();

    let response: hoshiguma_api::cooler::Response = send_command(
        &mut stream,
        hoshiguma_api::cooler::Request::SetCompressorState(
            hoshiguma_api::cooler::CompressorState::Run,
        ),
    );
    println!("Response: {:?}", response);
}

fn send_command<Req: Serialize, Resp: DeserializeOwned>(
    stream: &mut TcpStream,
    command: Req,
) -> Resp {
    println!("Sending command");
    let bytes = postcard::to_stdvec_cobs(&command).unwrap();
    stream.write_all(&bytes).unwrap();
    let request_time = std::time::Instant::now();

    println!("Waiting for response...");
    let mut bytes = Vec::new();
    let n = stream.read(&mut bytes).unwrap();
    let bytes = &mut bytes[..n];
    println!("bytes: {bytes:?}");
    let response = postcard::from_bytes_cobs(bytes).unwrap();
    let response_time = std::time::Instant::now();
    let duration = response_time.duration_since(request_time);
    println!("Got response in {}ms", duration.as_millis());

    response
}
