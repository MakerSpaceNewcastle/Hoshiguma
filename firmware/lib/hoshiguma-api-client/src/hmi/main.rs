use hoshiguma_api::{
    CONTROL_PORT, HMI_IP_ADDRESS,
    hmi::{Request, Response},
};
use hoshiguma_api_client::send_command;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((HMI_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(&mut stream, Request::GetGitRevision).await;
            send_command::<_, Response>(&mut stream, Request::GetBootReason).await;
            send_command::<_, Response>(&mut stream, Request::GetUptime).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let _ = tokio::join!(a);
}
