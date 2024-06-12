use clap::Parser;
use std::{io::Read, time::Duration};
use telemetry_protocols;
use tracing::{debug, error, info, warn};

/// Tool to receive data from coprocessors via the postcard protocol.
#[derive(Parser)]
struct Cli {
    /// Serial port
    #[arg(short, long)]
    port: String,

    /// Serial baud rate
    #[arg(short, long, default_value = "57600")]
    baud: u32,
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let port = serialport::new(cli.port, cli.baud)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(port) => {
            let mut rx_buffer: Vec<u8> = vec![];

            for b in port.bytes().flatten() {
                rx_buffer.push(b);

                if b == 0 {
                    debug!("Received {} bytes: {:?}", rx_buffer.len(), rx_buffer);

                    try_parse_and_print_payload::<telemetry_protocols::koishi::Payload>(
                        &mut rx_buffer,
                    );

                    rx_buffer.clear();
                }
            }
        }
        Err(e) => {
            error!("Failed to open port: {}", e);
            ::std::process::exit(1);
        }
    }
}

fn try_parse_and_print_payload<P: for<'de> serde::de::Deserialize<'de> + std::fmt::Debug>(
    rx_buffer: &mut [u8],
) {
    match postcard::from_bytes_cobs::<telemetry_protocols::Message<P>>(rx_buffer) {
        Ok(msg) => {
            info!("Received {:#?}", msg);
            if let telemetry_protocols::Payload::Boot(msg) = msg.payload {
                check_firmware_version(&msg);
            }
        }
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn check_firmware_version(msg: &telemetry_protocols::Boot) {
    let our_version = git_version::git_version!();
    let their_version = &msg.git_revision;

    debug!("Host Git revision: {}", our_version);
    debug!("Device Git revision: {}", their_version);

    if our_version != their_version {
        warn!("Git revisions do not match between host and device, this program may not read data correctly from the device!");
    }
}
