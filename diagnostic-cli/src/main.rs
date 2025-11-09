use clap::{Parser, Subcommand};
use log::error;
use teeny_rpc::transport::serialport::SerialTransport;

mod cooler;
mod extraction_airflow_sensor;
mod peripheral_controller;

/// Tool to receive data from coprocessors via the postcard protocol.
#[derive(Parser)]
struct Cli {
    /// Serial port
    #[arg(short, long)]
    port: String,

    /// Serial baud rate
    #[arg(short, long, default_value_t = 115_200)]
    baud: u32,

    #[command(subcommand)]
    device: Device,
}

trait Runner {
    async fn run(&self, transport: SerialTransport) -> Result<(), ()>;
}

#[derive(Subcommand)]
enum Device {
    PeripheralController(peripheral_controller::Cli),
    Cooler(cooler::Cli),
    ExtractionAirflowSensor(extraction_airflow_sensor::Cli),
}

impl Runner for Device {
    async fn run(&self, transport: SerialTransport) -> Result<(), ()> {
        match self {
            Device::PeripheralController(cli) => cli.run(transport).await,
            Device::Cooler(cli) => cli.run(transport).await,
            Device::ExtractionAirflowSensor(cli) => cli.run(transport).await,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    env_logger::init();

    let transport = SerialTransport::new(&cli.port, cli.baud).unwrap();

    if cli.device.run(transport).await.is_err() {
        error!("Command failed");
    }
}
