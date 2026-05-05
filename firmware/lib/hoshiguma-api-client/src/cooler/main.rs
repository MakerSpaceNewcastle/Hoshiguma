use hoshiguma_api::{
    CONTROL_PORT, COOLER_IP_ADDRESS,
    cooler::{CompressorState, Request, Response},
};
use hoshiguma_api_client::send_command;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(
                &mut stream,
                Request::SetCompressorState(CompressorState::Run),
            )
            .await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
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
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(&mut stream, Request::GetSystemInformation).await;
            send_command::<_, Response>(&mut stream, Request::GetCompressorState).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let c = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(&mut stream, Request::GetTemperatures).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        }
    });

    let _ = tokio::join!(a, b, c);
}
