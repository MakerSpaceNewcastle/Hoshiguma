use hoshiguma_api::{
    CONTROL_PORT, REAR_SENSOR_BOARD_IP_ADDRESS,
    rear_sensor_board::{LightPattern, Request, Response, StatusLightSettings},
};
use hoshiguma_api_client::send_command;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    env_logger::init();

    let a = tokio::spawn(async {
        let settings = [
            StatusLightSettings {
                red: LightPattern::ON,
                amber: LightPattern::OFF,
                green: LightPattern::OFF,
            },
            StatusLightSettings {
                red: LightPattern::OFF,
                amber: LightPattern::ON,
                green: LightPattern::OFF,
            },
            StatusLightSettings {
                red: LightPattern::OFF,
                amber: LightPattern::OFF,
                green: LightPattern::ON,
            },
            StatusLightSettings {
                red: LightPattern::BLINK_1HZ,
                amber: LightPattern::BLINK_2HZ,
                green: LightPattern::OFF,
            },
        ];

        loop {
            for setting in settings.iter() {
                let mut stream = TcpStream::connect((REAR_SENSOR_BOARD_IP_ADDRESS, CONTROL_PORT))
                    .await
                    .unwrap();
                send_command::<_, Response>(&mut stream, Request::SetStatusLight(setting.clone()))
                    .await;
                drop(stream);

                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    });

    let b = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((REAR_SENSOR_BOARD_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(&mut stream, Request::GetSystemInformation).await;
            send_command::<_, Response>(&mut stream, Request::GetTemperatures).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let c = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((REAR_SENSOR_BOARD_IP_ADDRESS, CONTROL_PORT))
                .await
                .unwrap();
            send_command::<_, Response>(&mut stream, Request::GetExtractionAirflow).await;
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let _ = tokio::join!(a, b, c);
}
