use crate::metric::Metric;
use core::sync::atomic::Ordering;
use defmt::{debug, info, warn, Format};
use embassy_futures::select::{select, Either};
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use embassy_rp::clocks::RoscRng;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Ticker};
use heapless::String;
use portable_atomic::AtomicU64;
use rand::RngCore;
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    headers::ContentType,
    request::{Method, RequestBuilder},
    response::StatusCode,
};

pub(crate) static METRIC_TX: PubSubChannel<CriticalSectionRawMutex, Metric, 32, 1, 2> =
    PubSubChannel::new();

pub(crate) static TELEMETRY_TX_BUFFER_SUBMISSIONS: AtomicU64 = AtomicU64::new(0);
pub(crate) static TELEMETRY_TX_SUCCESS: AtomicU64 = AtomicU64::new(0);
pub(crate) static TELEMETRY_TX_FAIL_BUFFER: AtomicU64 = AtomicU64::new(0);
pub(crate) static TELEMETRY_TX_FAIL_NETWORK: AtomicU64 = AtomicU64::new(0);

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("telem tx").await;

    let mut rng = RoscRng;
    let seed = rng.next_u64();

    let mut rx_buffer = [0; 8192];
    let mut tls_read_buffer = [0; 16640];
    let mut tls_write_buffer = [0; 16640];

    let client_state = TcpClientState::<1, 1024, 1024>::new();
    let tcp_client = TcpClient::new(stack, &client_state);
    let dns_client = DnsSocket::new(stack);
    let tls_config = TlsConfig::new(
        seed,
        &mut tls_read_buffer,
        &mut tls_write_buffer,
        TlsVerify::None,
    );

    let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

    let mut metric_rx = METRIC_TX.subscriber().unwrap();
    let mut purge_tick = Ticker::every(Duration::from_secs(2));

    let mut metric_buffer = MetricBuffer::default();

    loop {
        match select(metric_rx.next_message(), purge_tick.next()).await {
            Either::First(WaitResult::Message(metric)) => {
                // Add the metric to the buffer
                match metric_buffer.push(metric) {
                    Ok(_) => {
                        TELEMETRY_TX_BUFFER_SUBMISSIONS.add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        warn!("Failed to push metric to buffer");
                        TELEMETRY_TX_FAIL_BUFFER.add(1, Ordering::Relaxed);
                    }
                }

                // If the buffer is nearing capacity, then send now
                if metric_buffer.send_required() {
                    info!("Tx reason: buffer nearly full");
                    metric_buffer.tx(&mut http_client, &mut rx_buffer).await;
                }
            }
            Either::First(WaitResult::Lagged(_)) => unreachable!(),
            Either::Second(_) => {
                info!("Tx reason: periodic purge");
                metric_buffer.tx(&mut http_client, &mut rx_buffer).await;
            }
        }
    }
}

#[derive(Format, Default)]
pub(super) struct MetricBuffer {
    body: String<12288>,
}

pub(crate) const BUFFER_FREE_SPACE_THRESHOLD: usize = 2048;

impl MetricBuffer {
    fn push(&mut self, metric: Metric) -> Result<(), ()> {
        debug!("buffer length = {}", self.body.len());
        metric.format_influx(&mut self.body).map_err(|_| ())?;
        debug!("new buffer length = {}", self.body.len());
        Ok(())
    }

    fn send_required(&self) -> bool {
        let free = self.body.capacity() - self.body.len();
        free < BUFFER_FREE_SPACE_THRESHOLD
    }

    async fn tx<T: embedded_nal_async::TcpConnect, D: embedded_nal_async::Dns>(
        &mut self,
        http_client: &mut HttpClient<'_, T, D>,
        rx_buffer: &mut [u8],
    ) {
        if self.body.is_empty() {
            // Buffer is empty, nothing to do
            return;
        }

        const TELEGRAF_URL: &str = env!("TELEGRAF_URL");
        const TELEGRAF_USERNAME: &str = env!("TELEGRAF_USERNAME");
        const TELEGRAF_PASSWORD: &str = env!("TELEGRAF_PASSWORD");

        debug!("Submitting metrics to {}", &TELEGRAF_URL);

        let mut request = match http_client.request(Method::POST, TELEGRAF_URL).await {
            Ok(request) => request
                .basic_auth(TELEGRAF_USERNAME, TELEGRAF_PASSWORD)
                .content_type(ContentType::TextPlain)
                .body(self.body.as_bytes()),
            Err(e) => {
                warn!("Metrics submission failed: {}", e);
                TELEMETRY_TX_FAIL_NETWORK.add(1, Ordering::Relaxed);
                return;
            }
        };

        match request.send(rx_buffer).await {
            Ok(response) => {
                if response.status == StatusCode(204) {
                    debug!("Metrics submission success: status={}", response.status);
                } else {
                    warn!("Metrics submission failed: status={}", response.status);
                    TELEMETRY_TX_FAIL_NETWORK.add(1, Ordering::Relaxed);
                    return;
                }
            }
            Err(e) => {
                warn!("Metrics submission failed: {}", e);
                TELEMETRY_TX_FAIL_NETWORK.add(1, Ordering::Relaxed);
                return;
            }
        };

        // Clear the buffer once transmitted
        self.body.clear();

        debug!("Metric submission successful");
        TELEMETRY_TX_SUCCESS.add(1, Ordering::Relaxed);
    }
}
