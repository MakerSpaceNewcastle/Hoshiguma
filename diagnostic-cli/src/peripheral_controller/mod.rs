mod dashboard;

use crate::Runner;
use clap::{Parser, Subcommand};
use hoshiguma_protocol::peripheral_controller::rpc::{Request, Response};
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
    Dashboard,
    EventStream,

    Ping,
    GetSystemInformation,

    GetEventCount,
    GetEventStatistics,
    GetOldestEvent,
}

impl Runner for Cli {
    async fn run(&self, transport: SerialTransport) -> Result<(), ()> {
        let mut client = Client::<_, Request, Response>::new(transport, Duration::from_millis(100));
        let timeout = Duration::from_millis(self.timeout);

        match &self.command {
            Command::Dashboard => {
                dashboard::run(client).await.unwrap();
                Ok(())
            }
            Command::EventStream => {
                env_logger::init();

                let mut ticker =
                    tokio::time::interval(Duration::from_millis(self.repeat.unwrap_or(50)));

                loop {
                    match client.call(Request::GetOldestEvent, timeout).await {
                        Ok(Response::GetOldestEvent(Some(event))) => info!("{:#?}", event),
                        Ok(Response::GetOldestEvent(None)) => {}
                        Ok(response) => warn!("Unexpected response {:?}", response),
                        Err(e) => warn!("Command failed: {e}"),
                    }

                    ticker.tick().await;
                }
            }
            command => {
                env_logger::init();

                let mut ticker = tokio::time::interval(match self.repeat {
                    Some(ms) => Duration::from_millis(ms),
                    None => Duration::MAX,
                });

                loop {
                    let request = match command {
                        Command::Dashboard => unreachable!(),
                        Command::EventStream => unreachable!(),
                        Command::Ping => Request::Ping(42),
                        Command::GetSystemInformation => Request::GetSystemInformation,
                        Command::GetEventCount => Request::GetEventCount,
                        Command::GetEventStatistics => Request::GetEventStatistics,
                        Command::GetOldestEvent => Request::GetOldestEvent,
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
    }
}
