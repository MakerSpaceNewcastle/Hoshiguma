use hoshiguma_api::{
    API_PORT, REAR_SENSOR_BOARD_IP_ADDRESS,
    rear_sensor_board::{LightPattern, StatusLightSettings, request},
};
use hoshiguma_api_client::send_request;
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
                let mut stream = TcpStream::connect((REAR_SENSOR_BOARD_IP_ADDRESS, API_PORT))
                    .await
                    .unwrap();
                send_request(&mut stream, request::SetStatusLight(setting.clone()))
                    .await
                    .unwrap();
                drop(stream);

                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    });

    let b = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((REAR_SENSOR_BOARD_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(&mut stream, request::GetSystemInformation)
                .await
                .unwrap();
            send_request(&mut stream, request::GetTemperatures)
                .await
                .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let c = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((REAR_SENSOR_BOARD_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(&mut stream, request::GetExtractionAirflow)
                .await
                .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let _ = tokio::join!(a, b, c);
}
