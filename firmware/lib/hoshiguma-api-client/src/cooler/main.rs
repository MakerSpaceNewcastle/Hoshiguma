use hoshiguma_api::cooler::{CompressorState, Request, Response};
use hoshiguma_api_client::send_command;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    const ADDR: &str = "10.69.69.5:2000";

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
