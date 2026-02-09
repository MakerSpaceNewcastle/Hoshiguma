use hoshiguma_api::{
    API_PORT, COOLER_IP_ADDRESS,
    cooler::{CompressorState, request},
};
use hoshiguma_api_client::send_request;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(
                &mut stream,
                request::SetCompressorState(CompressorState::Run),
            )
            .await
            .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(
                &mut stream,
                request::SetCompressorState(CompressorState::Idle),
            )
            .await
            .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let b = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(&mut stream, request::GetSystemInformation)
                .await
                .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let c = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(&mut stream, request::GetTemperatures)
                .await
                .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        }
    });

    let _ = tokio::join!(a, b, c);
}
