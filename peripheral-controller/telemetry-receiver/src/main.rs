use clap::{Parser, Subcommand};
use hoshiguma_foundational_data::{
    koishi::Payload as KoishiPayload, satori::Payload as SatoriPayload, Boot, Message, Payload,
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
    #[arg(short, long, default_value = "57600")]
    baud: u32,

    #[command(subcommand)]
    payload_kind: PayloadKind,
}

#[derive(Subcommand)]
enum PayloadKind {
    Satori,
    Koishi,
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

                    if let Some(msg) = match cli.payload_kind {
                        PayloadKind::Koishi => {
                            try_parse_payload::<KoishiPayload>(rx_buffer.to_vec())
                        }
                        PayloadKind::Satori => {
                            try_parse_payload::<SatoriPayload>(rx_buffer.to_vec())
                        }
                    } {
                        info!("Received:\n{:#?}", msg);
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

fn try_parse_payload<
    P: for<'de> serde::de::Deserialize<'de> + std::fmt::Debug + Clone + 'static,
>(
    mut rx_buffer: Vec<u8>,
) -> Option<Box<dyn std::fmt::Debug>> {
    debug!("Receive buffer: {:?} (len {})", rx_buffer, rx_buffer.len());

    match postcard::from_bytes_cobs::<Message<P>>(&mut rx_buffer) {
        Ok(msg) => {
            if let Payload::Boot(ref msg) = msg.payload {
                check_firmware_version(msg);
            }
            Some(Box::new(msg))
        }
        Err(e) => {
            warn!("Failed to parse message: {e}");
            None
        }
    }
}

fn check_firmware_version(msg: &Boot) {
    let our_version = git_version::git_version!();
    let their_version = &msg.git_revision;

    debug!("Host Git revision: {}", our_version);
    debug!("Device Git revision: {}", their_version);

    if our_version != their_version {
        warn!("Git revisions do not match between host and device, this program may not read data correctly from the device!");
    }
}
