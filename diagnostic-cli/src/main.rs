use std::time::Duration;

use clap::{Parser, ValueEnum};
use hoshiguma_protocol::{
    peripheral_controller::rpc::{Request, Response},
    types::SystemInformation,
};
use teeny_rpc::{client::Client, transport::serialport::SerialTransport};
use tracing::{debug, info, warn};

/// Tool to receive data from coprocessors via the postcard protocol.
#[derive(Parser)]
struct Cli {
    /// Serial port
    #[arg(short, long)]
    port: String,

    /// Serial baud rate
    #[arg(short, long, default_value_t = 115_200)]
    baud: u32,

    /// Format to print received messages in
    #[arg(short, long, default_value = "debug-pretty")]
    format: PrintFormat,
}

#[derive(Clone, ValueEnum)]
enum PrintFormat {
    Debug,
    DebugPretty,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let transport = SerialTransport::new(&cli.port, cli.baud).unwrap();
    let mut client = Client::<_, Request, Response>::new(transport);

    const TIMEOUT: Duration = Duration::from_millis(50);

    let info = client
        .call(Request::GetSystemInformation, TIMEOUT)
        .await
        .unwrap();
    if let Response::GetSystemInformation(info) = info {
        info!("Device: {:?}", info);
        check_firmware_version(&info);
    } else {
        panic!("Incorrect response from request");
    }

    let mut ticker = tokio::time::interval(Duration::from_millis(200));

    'telem_rx: loop {
        match client.call(Request::GetOldestEvent, TIMEOUT).await {
            Ok(Response::GetOldestEvent(Some(event))) => {
                match cli.format {
                    PrintFormat::Debug => println!("{:?}", event),
                    PrintFormat::DebugPretty => info!("Received:\n{:#?}", event),
                }

                // Immediately request further events
                ticker.reset();
                continue 'telem_rx;
            }
            Ok(Response::GetOldestEvent(None)) => {
                // Do nothing
            }
            Ok(_) => warn!("Incorrect response from request"),
            Err(e) => warn!("Call error: {e}"),
        }

        ticker.tick().await;
    }
}

fn check_firmware_version(info: &SystemInformation) {
    let our_version = git_version::git_version!();
    let their_version = &info.git_revision;

    debug!("Host Git revision: {}", our_version);
    debug!("Device Git revision: {}", their_version);

    if our_version != their_version {
        warn!("Git revisions do not match between host and device, this program may not read data correctly from the device!");
    }
}
