use clap::Parser;
use hoshiguma_api::{
    API_PORT, COOLER_IP_ADDRESS,
    cooler::{CompressorState, CoolantPumpState, RadiatorFanState, request},
};
use hoshiguma_api_client::send_request;
use log::{info, warn};
use tokio::net::TcpStream;

/// Operate the cooler independently of the main orchestrator, maintaining a
/// target temperature by controlling the compressor.
#[derive(Debug, Parser)]
struct Args {
    /// Temperature setpoint in degrees Celsius
    #[arg(long)]
    setpoint: f32,

    /// Hysteresis half-band in degrees Celsius. The compressor turns on above
    /// `setpoint + hysteresis` and off below `setpoint - hysteresis`.
    #[arg(long, default_value_t = 1.0)]
    hysteresis: f32,

    /// 1-Wire address of the reservoir temperature sensor (decimal or 0x-prefixed hex)
    #[arg(long, value_parser = parse_sensor_id)]
    sensor_id: u64,
}

fn parse_sensor_id(s: &str) -> Result<u64, String> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        s.parse::<u64>().map_err(|e| e.to_string())
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    info!(
        "Starting independent cooler control: setpoint={:.1}°C, hysteresis=±{:.1}°C, sensor=0x{:016X}",
        args.setpoint, args.hysteresis, args.sensor_id
    );

    // Turn on pump and radiator fan, leave compressor idle at startup.
    startup_pump().await;
    startup_fan().await;
    startup_compressor_idle().await;

    // Watchdog task: keeps the cooler's watchdog timer alive by polling system
    // information and coolant rates every second.
    let watchdog = tokio::spawn(async {
        loop {
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            send_request(&mut stream, request::GetSystemInformation)
                .await
                .unwrap();
            send_request(&mut stream, request::GetCoolantFlowRate)
                .await
                .unwrap();
            send_request(&mut stream, request::GetCoolantReturnRate)
                .await
                .unwrap();
            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    // Temperature control task: bang-bang compressor control every 5 seconds.
    let control = tokio::spawn(async move {
        let mut compressor_on = false;

        loop {
            let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
                .await
                .unwrap();
            let temps = send_request(&mut stream, request::GetTemperatures)
                .await
                .unwrap();

            match temps.0 {
                Err(()) => {
                    warn!("Cooler reported an error reading temperatures");
                }
                Ok(readings) => {
                    match readings.into_iter().find(|r| r.address == args.sensor_id) {
                        None => {
                            warn!(
                                "Sensor 0x{:016X} not found in temperature readings",
                                args.sensor_id,
                            );
                        }
                        Some(reading) => match reading.reading {
                            Err(()) => {
                                warn!("Sensor 0x{:016X} returned an error reading", args.sensor_id);
                            }
                            Ok(temp) => {
                                info!("Reservoir temperature: {temp:.2}°C");

                                let new_state = if temp > args.setpoint + args.hysteresis {
                                    true
                                } else if temp < args.setpoint - args.hysteresis {
                                    false
                                } else {
                                    compressor_on // within band, hold current state
                                };

                                if new_state != compressor_on {
                                    compressor_on = new_state;
                                    let state = if compressor_on {
                                        info!(
                                            "Temperature {temp:.2}°C > {:.2}°C — compressor ON",
                                            args.setpoint + args.hysteresis
                                        );
                                        CompressorState::Run
                                    } else {
                                        info!(
                                            "Temperature {temp:.2}°C < {:.2}°C — compressor OFF",
                                            args.setpoint - args.hysteresis
                                        );
                                        CompressorState::Idle
                                    };

                                    let mut stream =
                                        TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
                                            .await
                                            .unwrap();
                                    send_request(&mut stream, request::SetCompressorState(state))
                                        .await
                                        .unwrap();
                                }
                            }
                        },
                    }
                }
            }

            drop(stream);

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let _ = tokio::join!(watchdog, control);
}

async fn startup_pump() {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(
        &mut stream,
        request::SetCoolantPumpState(CoolantPumpState::Run),
    )
    .await
    .unwrap();
}

async fn startup_fan() {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(
        &mut stream,
        request::SetRadiatorFanState(RadiatorFanState::Run),
    )
    .await
    .unwrap();
}

async fn startup_compressor_idle() {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(
        &mut stream,
        request::SetCompressorState(CompressorState::Idle),
    )
    .await
    .unwrap();
}
