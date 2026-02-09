use heapless::Vec;
use hoshiguma_api::{
    API_PORT, DesiredMachinePower, HMI_IP_ADDRESS, Interlock, MachineRun, Message, Severity,
    hmi::{AccessControlRawInput, OnscreenMessage, Screen, StatusScreenInfo, from_hmi, to_hmi},
};
use hoshiguma_api_client::{message_handler, send_request};
use log::{info, warn};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    env_logger::init();

    let a = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((HMI_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(&mut stream, to_hmi::request::GetSystemInformation)
                .await
                .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    let b = tokio::spawn(async {
        let listener = TcpListener::bind(("0.0.0.0", API_PORT)).await.unwrap();

        loop {
            let (mut stream, _) = listener.accept().await.unwrap();

            message_handler(&mut stream, |mut msg| {
                if msg
                    .payload::<from_hmi::request::NotifyPanelInteraction>()
                    .is_ok()
                {
                    info!("Notify: panel interaction");
                    Message::new(&from_hmi::response::AckPanelInteraction).unwrap()
                } else if let Ok(payload) =
                    msg.payload::<from_hmi::request::NotifyAccessControlInputChanged>()
                {
                    info!("Notify: access control inputs changed: {payload:?}");
                    Message::new(&from_hmi::response::AckAccessControlInputChanged(payload.0))
                        .unwrap()
                } else if let Ok(payload) =
                    msg.payload::<from_hmi::request::NotifyAccessControlStateChanged>()
                {
                    info!("Notify: access control state changed: {payload:?}");
                    Message::new(&from_hmi::response::AckAccessControlStateChanged(payload.0))
                        .unwrap()
                } else {
                    warn!("Notify: unknown message type");
                    Message::new(&from_hmi::response::ApiError).unwrap()
                }
            })
            .await;
        }
    });

    let c = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((HMI_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(
                &mut stream,
                to_hmi::request::SetStatusScreenInfo(StatusScreenInfo {
                    access_control: AccessControlRawInput::Denied,
                    machine_power: DesiredMachinePower::On,
                    interlock: Interlock::OperationPermitted,
                    running: MachineRun::Idle,
                    messages: Vec::from([
                        OnscreenMessage {
                            text: "I'm information".try_into().unwrap(),
                            severity: Severity::Information,
                        },
                        OnscreenMessage {
                            text: "I'm normal".try_into().unwrap(),
                            severity: Severity::Normal,
                        },
                        OnscreenMessage {
                            text: "I'm warning".try_into().unwrap(),
                            severity: Severity::Warning,
                        },
                        OnscreenMessage {
                            text: "I'm critical".try_into().unwrap(),
                            severity: Severity::Critical,
                        },
                        OnscreenMessage {
                            text: "I'm fatal".try_into().unwrap(),
                            severity: Severity::Fatal,
                        },
                    ]),
                }),
            )
            .await
            .unwrap();
            send_request(&mut stream, to_hmi::request::BacklightWake)
                .await
                .unwrap();
            send_request(&mut stream, to_hmi::request::ShowScreen(Screen::Status))
                .await
                .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let mut stream = TcpStream::connect((HMI_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(
                &mut stream,
                to_hmi::request::SetStatusScreenInfo(StatusScreenInfo {
                    access_control: AccessControlRawInput::Denied,
                    machine_power: DesiredMachinePower::On,
                    interlock: Interlock::OperationPermitted,
                    running: MachineRun::Idle,
                    messages: Vec::from([
                        OnscreenMessage {
                            text: "Telemetry INOP".try_into().unwrap(),
                            severity: Severity::Information,
                        },
                        OnscreenMessage {
                            text: "AC Bus Off".try_into().unwrap(),
                            severity: Severity::Critical,
                        },
                        OnscreenMessage {
                            text: "Extraction Airflow Low".try_into().unwrap(),
                            severity: Severity::Warning,
                        },
                        OnscreenMessage {
                            text: "Door(s) Open".try_into().unwrap(),
                            severity: Severity::Critical,
                        },
                        OnscreenMessage {
                            text: "Coolant Rate Asymmetry".try_into().unwrap(),
                            severity: Severity::Fatal,
                        },
                        OnscreenMessage {
                            text: "Temperature Sensor Fault".try_into().unwrap(),
                            severity: Severity::Warning,
                        },
                    ]),
                }),
            )
            .await
            .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    let _ = tokio::join!(a, b, c);
}
