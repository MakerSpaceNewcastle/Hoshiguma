use hoshiguma_api::hmi::{Request, Response};
use hoshiguma_api_client::send_command;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    const ADDR: &str = "10.69.69.4:2000";

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect(ADDR).await.unwrap();
            send_command::<_, Response>(&mut stream, Request::GetGitRevision).await;
            send_command::<_, Response>(&mut stream, Request::GetBootReason).await;
            send_command::<_, Response>(&mut stream, Request::GetUptime).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let _ = tokio::join!(a);
}
