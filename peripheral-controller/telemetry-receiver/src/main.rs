use clap::{Parser, ValueEnum};
use hoshiguma_telemetry_protocol::{
    payload::{
        system::{Boot, SystemMessagePayload},
        Payload,
    },
    Message,
};
use std::{io::Read, time::Duration};
use tracing::{debug, error, info, warn};

/// Tool to receive data from coprocessors via the postcard protocol.
#[derive(Parser)]
struct Cli {
    /// Serial port
    #[arg(short, long)]
    port: String,

    /// Serial baud rate
    #[arg(short, long, default_value = "115200")]
    baud: u32,

    /// Format to print received messages in
    #[arg(short, long, default_value = "debug-pretty")]
    format: PrintFormat,
}

#[derive(Clone, ValueEnum)]
enum PrintFormat {
    Debug,
    DebugPretty,
    Json,
    JsonPretty,
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
                    debug!("Receive buffer: {:?} (len {})", rx_buffer, rx_buffer.len());

                    match postcard::from_bytes_cobs::<Message>(&mut rx_buffer) {
                        Ok(msg) => {
                            if let Payload::System(SystemMessagePayload::Boot(ref msg)) =
                                msg.payload
                            {
                                check_firmware_version(msg);
                            }

                            match cli.format {
                                PrintFormat::Debug => println!("{:?}", msg),
                                PrintFormat::DebugPretty => info!("Received:\n{:#?}", msg),
                                PrintFormat::Json => match serde_json::to_string(&msg) {
                                    Ok(s) => println!("{s}"),
                                    Err(e) => warn!("Failed to JSON serialise message: {e}"),
                                },
                                PrintFormat::JsonPretty => {
                                    match serde_json::to_string_pretty(&msg) {
                                        Ok(s) => println!("{s}"),
                                        Err(e) => warn!("Failed to JSON serialise message: {e}"),
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {e}");
                        }
                    }

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

fn check_firmware_version(msg: &Boot) {
    #[cfg(feature = "git-version")]
    let our_version = git_version::git_version!();
    #[cfg(not(feature = "git-version"))]
    let our_version = "unknown";

    let their_version = &msg.git_revision;

    debug!("Host Git revision: {}", our_version);
    debug!("Device Git revision: {}", their_version);

    if our_version != their_version {
        warn!("Git revisions do not match between host and device, this program may not read data correctly from the device!");
    }
}
