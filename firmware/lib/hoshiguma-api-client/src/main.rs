use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    let response: hoshiguma_api::cooler::Response =
        send_command(&mut stream, hoshiguma_api::cooler::Request::GetTemperatures);
    println!("Response: {:?}", response);
}

fn send_command<Req: Serialize, Resp: DeserializeOwned>(
    stream: &mut TcpStream,
    command: Req,
) -> Resp {
    println!("Sending command");
    let bytes = postcard::to_stdvec(&command).unwrap();
    stream.write_all(&bytes).unwrap();
    let request_time = std::time::Instant::now();

    println!("Waiting for response...");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let response = postcard::from_bytes(&bytes).unwrap();
    let response_time = std::time::Instant::now();
    let duration = response_time.duration_since(request_time);
    println!("Got response in {}ms", duration.as_millis());

    response
}
