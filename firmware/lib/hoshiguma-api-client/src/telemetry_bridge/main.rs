use hoshiguma_api::{
    CONTROL_PORT, TELEMETRY_MODULE_IP_ADDRESS,
    telemetry_bridge::{Request, Response},
};
use hoshiguma_api_client::send_command;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((TELEMETRY_MODULE_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(&mut stream, Request::IsReady).await;
            send_command::<_, Response>(&mut stream, Request::GetTime).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let _ = tokio::join!(a);
}
