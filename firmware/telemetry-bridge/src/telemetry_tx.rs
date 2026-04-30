use crate::{
    api::NUM_LISTENERS,
    self_telemetry::{DATA_POINTS_ACCEPTED, DATA_POINTS_DISCARDED},
    telegraf_buffer::TelegrafBuffer,
};
use core::sync::atomic::Ordering;
use defmt::{debug, warn};
use embassy_futures::select::{Either, select};
use embassy_net::{
    Stack,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use embassy_rp::clocks::RoscRng;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Instant, Timer};
use hoshiguma_api::telemetry_bridge::FormattedTelemetryDataPoint;
use reqwless::client::{HttpClient, TlsConfig, TlsVerify};

const TELEMETRY_PUBLISHERS: usize = NUM_LISTENERS + 1;

pub(crate) static TELEMETRY_TX: PubSubChannel<
    CriticalSectionRawMutex,
    FormattedTelemetryDataPoint,
    64,
    1,
    TELEMETRY_PUBLISHERS,
> = PubSubChannel::new();

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) {
    let mut rng = RoscRng;

    'connection: loop {
        let mut rx_buffer = [0; 8192];
        let mut tls_read_buffer = [0; 16640];
        let mut tls_write_buffer = [0; 16640];

        let client_state = TcpClientState::<1, 1024, 1024>::new();
        let tcp_client = TcpClient::new(stack, &client_state);
        let dns_client = DnsSocket::new(stack);
        let tls_config = TlsConfig::new(
            rng.next_u64(),
            &mut tls_read_buffer,
            &mut tls_write_buffer,
            TlsVerify::None,
        );

        let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

        let mut data_point_line_rx = TELEMETRY_TX.subscriber().unwrap();

        let mut telegraf_buffer = TelegrafBuffer::default();

        const TX_INTERVAL: Duration = Duration::from_millis(800);
        let mut next_tx = Instant::now() + TX_INTERVAL;

        loop {
            match select(data_point_line_rx.next_message(), Timer::at(next_tx)).await {
                Either::First(WaitResult::Message(data_point)) => {
                    // Add the data point to the buffer
                    match telegraf_buffer.push(data_point.0) {
                        Ok(_) => {
                            DATA_POINTS_ACCEPTED.add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            warn!("Failed to push metric to buffer");
                            DATA_POINTS_DISCARDED.add(1, Ordering::Relaxed);
                        }
                    }

                    // If the buffer is nearing capacity, then send now
                    if telegraf_buffer.send_required() {
                        warn!("Scheduling immediate send due to buffer capacity");
                        next_tx = Instant::now();
                    }
                }
                Either::First(WaitResult::Lagged(n)) => {
                    warn!("Subscriber lagged, lost {} messages", n);
                }
                Either::Second(_) => {
                    debug!("Submitting buffered telemetry data");
                    telegraf_buffer.tx(&mut http_client, &mut rx_buffer).await;

                    if !stack.is_config_up() {
                        warn!("Network down");
                        continue 'connection;
                    }

                    next_tx = Instant::now() + TX_INTERVAL;
                }
            }
        }
    }
}
