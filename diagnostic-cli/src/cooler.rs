use crate::Runner;
use clap::{Parser, Subcommand};
use hoshiguma_protocol::cooler::{
    rpc::{Request, Response},
    types::{CompressorState, CoolantPumpState, RadiatorFanState, StirrerState},
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
    SetStirrerOff,
    SetStirrerOn,
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
                Command::Ping => Request::Ping(42),
                Command::GetSystemInformation => Request::GetSystemInformation,
                Command::GetState => Request::GetState,
                Command::SetRadiatorFanOff => Request::SetRadiatorFan(RadiatorFanState::Idle),
                Command::SetRadiatorFanOn => Request::SetRadiatorFan(RadiatorFanState::Run),
                Command::SetCompressorOff => Request::SetCompressor(CompressorState::Idle),
                Command::SetCompressorOn => Request::SetCompressor(CompressorState::Run),
                Command::SetStirrerOff => Request::SetStirrer(StirrerState::Idle),
                Command::SetStirrerOn => Request::SetStirrer(StirrerState::Run),
                Command::SetCoolantPumpOff => Request::SetCoolantPump(CoolantPumpState::Idle),
                Command::SetCoolantPumpOn => Request::SetCoolantPump(CoolantPumpState::Run),
            };

            match client.call(request, timeout).await {
                Ok(response) => info!("Response: {:#?}", response),
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
