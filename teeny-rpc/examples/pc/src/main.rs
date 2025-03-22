use clap::{Parser, Subcommand};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use teeny_rpc::{client::Client, server::Server, transport::serialport::SerialTransport};

#[derive(Parser)]
#[command()]
struct Cli {
    #[arg(short, long)]
    port: String,

    #[arg(short, long, default_value_t = 115_200)]
    baud: u32,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Client,
    Server,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum Request {
    Ping(u32),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum Response {
    Ping(u32),
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let transport = SerialTransport::new(&cli.port, cli.baud).unwrap();

    match &cli.command {
        Commands::Client => {
            info!("Running client");
            let mut client = Client::<_, Request, Response>::new(transport);

            let mut i = 0;
            loop {
                match client.call(Request::Ping(i), Duration::from_secs(1)).await {
                    Ok(response) => {
                        info!("Client got response: {:?}", response);
                    }
                    Err(e) => {
                        warn!("Client failed making request: {e}");
                    }
                }

                i = i.wrapping_add(1);

                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
        Commands::Server => {
            info!("Running server");
            let mut server = Server::<_, Request, Response>::new(transport);

            loop {
                match server.wait_for_request(Duration::from_secs(5)).await {
                    Ok(request) => {
                        let response = match request {
                            Request::Ping(i) => Response::Ping(i),
                        };
                        if let Err(e) = server.send_response(response).await {
                            warn!("Server failed sending response: {e}");
                        }
                    }
                    Err(e) => {
                        warn!("Server failed waiting for request: {e}");
                    }
                }
            }
        }
    }
}
