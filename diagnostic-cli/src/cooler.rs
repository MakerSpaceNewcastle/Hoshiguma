use crate::Runner;
use clap::{Parser, Subcommand};
use hoshiguma_protocol::accessories::{
    cooler::{
        rpc::Request as CoolerRequest,
        types::{CompressorState, CoolantPumpState, RadiatorFanState},
    },
    rpc::{Request, Response},
};
use log::{info, warn};
use std::time::Duration;
use teeny_rpc::{client::Client, transport::serialport::SerialTransport};

#[derive(Parser)]
pub(super) struct Cli {
    /// RPC request timeout
    #[arg(long, default_value_t = 50)]
    timeout: u64,

    /// Repeat the command every n milliseconds
    #[arg(long, default_value = None)]
    repeat: Option<u64>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Ping,
    GetSystemInformation,

    GetState,

    SetRadiatorFanOff,
    SetRadiatorFanOn,
    SetCompressorOff,
    SetCompressorOn,
    SetCoolantPumpOff,
    SetCoolantPumpOn,
}

impl Runner for Cli {
    async fn run(&self, transport: SerialTransport) -> Result<(), ()> {
        let mut client = Client::<_, Request, Response>::new(transport, Duration::from_millis(100));
        let timeout = Duration::from_millis(self.timeout);

        let mut ticker = tokio::time::interval(match self.repeat {
            Some(ms) => Duration::from_millis(ms),
            None => Duration::MAX,
        });

        loop {
            let request = match self.command {
                Command::Ping => CoolerRequest::Ping(42),
                Command::GetSystemInformation => CoolerRequest::GetSystemInformation,
                Command::GetState => CoolerRequest::GetState,
                Command::SetRadiatorFanOff => CoolerRequest::SetRadiatorFan(RadiatorFanState::Idle),
                Command::SetRadiatorFanOn => CoolerRequest::SetRadiatorFan(RadiatorFanState::Run),
                Command::SetCompressorOff => CoolerRequest::SetCompressor(CompressorState::Idle),
                Command::SetCompressorOn => CoolerRequest::SetCompressor(CompressorState::Run),
                Command::SetCoolantPumpOff => CoolerRequest::SetCoolantPump(CoolantPumpState::Idle),
                Command::SetCoolantPumpOn => CoolerRequest::SetCoolantPump(CoolantPumpState::Run),
            };

            match client.call(Request::Cooler(request), timeout).await {
                Ok(response) => info!("Response: {response:#?}"),
                Err(e) => warn!("Command failed: {e}"),
            }

            if self.repeat.is_none() {
                break Ok(());
            } else {
                ticker.tick().await;
            }
        }
    }
}
