use clap::Parser;
use hoshiguma_api::{
    API_PORT, COOLER_IP_ADDRESS,
    cooler::{CoolantPumpState, RawCoolantRate, request},
};
use hoshiguma_api_client::send_request;
use log::{info, warn};
use std::io::Write as _;
use std::time::Duration;
use tokio::{net::TcpStream, time::Instant};

/// Cooler coolant flow and return rate sensor utilities.
///
/// Defaults to the `calibrate` sub-command when none is given.
#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Debug, clap::Subcommand)]
enum Mode {
    /// Calibrate the cooler's coolant flow and return rate sensors (default).
    ///
    /// Waits for the user to press Enter, waits for both sensors to read zero,
    /// records the baseline pulse counts, runs the pump until Enter is pressed
    /// again, waits for zero, records the final pulse counts, then asks how
    /// many litres were moved to compute pulses-per-litre for each sensor.
    Calibrate {
        /// Seconds both sensors must continuously report zero before a pulse
        /// snapshot is taken
        #[arg(long, default_value_t = 5)]
        settle_time: u64,
    },

    /// Turn on the pump and continuously report converted coolant rates
    Test {
        /// Pulses-per-litre calibration value for the flow sensor
        #[arg(long)]
        flow_ppl: f64,

        /// Pulses-per-litre calibration value for the return sensor
        #[arg(long)]
        return_ppl: f64,
    },
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Calibrate { settle_time: 5 }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    // Keep the cooler's watchdog timer alive for the entire lifetime of the
    // process by polling GetSystemInformation every 500 ms.
    tokio::spawn(async {
        loop {
            match TcpStream::connect((COOLER_IP_ADDRESS, API_PORT)).await {
                Ok(mut stream) => {
                    send_request(&mut stream, request::GetSystemInformation)
                        .await
                        .unwrap();
                }
                Err(e) => {
                    warn!("Watchdog: failed to connect: {e}");
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    match args.mode.unwrap_or_default() {
        Mode::Calibrate { settle_time } => run_calibration(Duration::from_secs(settle_time)).await,
        Mode::Test {
            flow_ppl,
            return_ppl,
        } => run_test_mode(flow_ppl, return_ppl).await,
    }
}

async fn run_calibration(settle_time: Duration) {
    println!("=== Coolant Rate Sensor Calibration ===");
    println!("Settle time: {}s", settle_time.as_secs());
    println!();

    // Phase 1 – wait for user to confirm ready, then wait for initial zero
    print!("Press Enter when ready to begin...");
    std::io::stdout().flush().unwrap();
    read_line().await;

    println!("Phase 1: waiting for both sensors to reach zero...");
    wait_for_zero(settle_time).await;

    let flow_start = get_flow_pulses().await;
    let return_start = get_return_pulses().await;
    info!("Baseline pulse counts — flow: {flow_start}, return: {return_start}");
    println!("  Baseline — flow: {flow_start} pulses, return: {return_start} pulses");

    // Phase 2 – run the pump until the user presses Enter
    println!();
    set_pump(CoolantPumpState::Run).await;
    println!("Phase 2: pump is running.");
    print!("Press Enter to stop the pump...");
    std::io::stdout().flush().unwrap();
    read_line().await;

    set_pump(CoolantPumpState::Idle).await;
    println!("  Pump stopped.");

    // Phase 3 – wait for final zero
    println!();
    println!("Phase 3: waiting for both sensors to reach zero...");
    wait_for_zero(settle_time).await;

    let flow_end = get_flow_pulses().await;
    let return_end = get_return_pulses().await;
    info!("Final pulse counts — flow: {flow_end}, return: {return_end}");
    println!("  Final     — flow: {flow_end} pulses, return: {return_end} pulses");

    let flow_delta = flow_end.saturating_sub(flow_start);
    let return_delta = return_end.saturating_sub(return_start);
    println!();
    println!("  Delta     — flow: {flow_delta} pulses, return: {return_delta} pulses");

    // Phase 4 – ask for the measured volume
    println!();
    print!("Enter the volume of coolant that was moved (litres): ");
    std::io::stdout().flush().unwrap();

    let line = read_line().await;
    let litres: f64 = line.trim().parse().expect("Expected a number");

    // Compute and print results
    let flow_ppl = flow_delta as f64 / litres;
    let return_ppl = return_delta as f64 / litres;

    println!();
    println!("=== Calibration Results ===");
    println!("Flow sensor:   {flow_delta} pulses / {litres:.4} L  =  {flow_ppl:.3} pulses/litre");
    println!(
        "Return sensor: {return_delta} pulses / {litres:.4} L  =  {return_ppl:.3} pulses/litre"
    );
}

async fn run_test_mode(flow_ppl: f64, return_ppl: f64) {
    println!("=== Coolant Rate Sensor Test Mode ===");
    println!("Flow PPL: {flow_ppl:.3}   Return PPL: {return_ppl:.3}");
    println!("Starting pump... (press Ctrl-C to exit — pump will remain running)");
    println!();

    set_pump(CoolantPumpState::Run).await;

    loop {
        let flow_rate = get_flow_rate().await;
        let return_rate = get_return_rate().await;

        let fmt_rate = |r: Result<RawCoolantRate, ()>, ppl: f64| match r {
            Ok(raw) => format!("{:.4} L/min", raw.into_rate(ppl).into_inner()),
            Err(()) => "sensor error".to_string(),
        };

        println!(
            "Flow: {:>16}  |  Return: {:>16}",
            fmt_rate(flow_rate, flow_ppl),
            fmt_rate(return_rate, return_ppl),
        );

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Read a line from stdin, returning the raw string.
async fn read_line() -> String {
    tokio::task::spawn_blocking(|| {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read stdin");
        buf
    })
    .await
    .unwrap()
}

/// Block until both the flow and return sensors have reported zero
/// continuously for `duration`.
async fn wait_for_zero(duration: Duration) {
    let mut zero_since: Option<Instant> = None;

    loop {
        let flow = get_flow_rate().await;
        let ret = get_return_rate().await;

        let flow_zero = flow.is_ok_and(|r| *r.pulses() == 0);
        let ret_zero = ret.is_ok_and(|r| *r.pulses() == 0);

        if flow_zero && ret_zero {
            let since = zero_since.get_or_insert_with(Instant::now);
            let elapsed = since.elapsed();
            print!(
                "\r  Both at zero for {:.1}s / {}s…   ",
                elapsed.as_secs_f64(),
                duration.as_secs()
            );
            std::io::stdout().flush().ok();
            if elapsed >= duration {
                println!();
                return;
            }
        } else {
            if zero_since.take().is_some() {
                print!("\r  Flow detected — resetting timer…        ");
                std::io::stdout().flush().ok();
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn set_pump(state: CoolantPumpState) {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(&mut stream, request::SetCoolantPumpState(state))
        .await
        .unwrap();
}

async fn get_flow_rate() -> Result<RawCoolantRate, ()> {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(&mut stream, request::GetCoolantFlowRate)
        .await
        .unwrap()
        .0
}

async fn get_return_rate() -> Result<RawCoolantRate, ()> {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(&mut stream, request::GetCoolantReturnRate)
        .await
        .unwrap()
        .0
}

async fn get_flow_pulses() -> u64 {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(&mut stream, request::GetCoolantFlowPulses)
        .await
        .unwrap()
        .0
        .unwrap()
}

async fn get_return_pulses() -> u64 {
    let mut stream = TcpStream::connect((COOLER_IP_ADDRESS, API_PORT))
        .await
        .unwrap();
    send_request(&mut stream, request::GetCoolantReturnPulses)
        .await
        .unwrap()
        .0
        .unwrap()
}
