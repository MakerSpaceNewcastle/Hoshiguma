use crate::{
    api::NUM_LISTENERS,
    self_telemetry::{DATA_POINTS_ACCEPTED, DATA_POINTS_DISCARDED},
    telegraf_buffer::TelegrafBuffer,
};
use core::sync::atomic::Ordering;
use defmt::{trace, warn};
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
use embassy_time::{Duration, Instant, Timer, WithTimeout};
use hoshiguma_api::telemetry_bridge::FormattedTelemetryDataPoint;
use portable_atomic::AtomicBool;
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::Method,
    response::StatusCode,
};

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

    let mut telegraf_buffer = TelegrafBuffer::default();

    'connection: loop {
        // Mark the connection as inoperative
        READY.store(false, Ordering::Relaxed);

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

        // Wait until we can actually contact Telegraf
        'initial_contact: loop {
            const TELEGRAF_URL: &str = env!("TELEGRAF_URL");

            match http_client
                .request(Method::GET, TELEGRAF_URL)
                .with_timeout(Duration::from_secs(2))
                .await
            {
                Ok(Ok(mut request)) => {
                    match request
                        .send(&mut rx_buffer)
                        .with_timeout(Duration::from_secs(2))
                        .await
                    {
                        Ok(Ok(response)) if response.status == StatusCode(401) => {
                            break 'initial_contact;
                        }
                        Ok(Ok(response)) => {
                            warn!("Failed to contact Telegraf: status={}", response.status);
                        }
                        Ok(Err(e)) => {
                            warn!("Failed to contact Telegraf: {}", e);
                        }
                        Err(_) => {
                            warn!("Failed to contact Telegraf: timeout");
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("Failed to contact Telegraf: {}", e);
                }
                Err(_) => {
                    warn!("Failed to contact Telegraf: timeout");
                }
            };

            // Wait before trying again
            Timer::after_secs(3).await;
        }
        READY.store(true, Ordering::Relaxed);

        let mut data_point_line_rx = TELEMETRY_TX.subscriber().unwrap();

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
                    trace!("Submitting buffered telemetry data");

                    if telegraf_buffer
                        .tx(&mut http_client, &mut rx_buffer)
                        .await
                        .is_err()
                    {
                        warn!("Network down");
                        continue 'connection;
                    }

                    next_tx = Instant::now() + TX_INTERVAL;
                }
            }
        }
    }
}

static READY: AtomicBool = AtomicBool::new(false);

pub(crate) fn is_ready() -> bool {
    READY.load(Ordering::Relaxed)
}
