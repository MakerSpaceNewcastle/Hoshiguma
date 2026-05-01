use hoshiguma_api::{
    CONTROL_PORT, TELEMETRY_BRIDGE_IP_ADDRESS,
    telemetry_bridge::{FormattedTelemetryDataPoint, Request, Response},
};
use hoshiguma_api_client::send_command;
use log::info;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((TELEMETRY_BRIDGE_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(&mut stream, Request::IsReady).await;
            send_command::<_, Response>(&mut stream, Request::GetTime).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let b = tokio::spawn(async {
        let mut data_point_count = 0usize;

        loop {
            let mut stream = TcpStream::connect((TELEMETRY_BRIDGE_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            for _ in 0..10 {
                send_command::<_, Response>(
                    &mut stream,
                    Request::SendTelemetryDataPoint(FormattedTelemetryDataPoint(
                        "some_data_point,with=lots,of=extra stuff=\"added\",number=42 1234567890"
                            .try_into()
                            .unwrap(),
                    )),
                )
                .await;
                data_point_count += 1;
            }
            drop(stream);

            info!("Number of telemetry points sent: {data_point_count}");

            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
        }
    });

    let _ = tokio::join!(a, b);
}
