use crate::metric::Metric;
use core::sync::atomic::Ordering;
use defmt::{debug, warn, Format};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::Duration;
use heapless::String;
use portable_atomic::AtomicU64;
use reqwless::{
    client::HttpClient,
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

#[derive(Format, Default)]
pub(super) struct MetricBuffer {
    body: String<12288>,
}

pub(crate) const BUFFER_FREE_SPACE_THRESHOLD: usize = 2048;

impl MetricBuffer {
    pub(super) fn push(&mut self, metric: Metric) -> Result<(), ()> {
        debug!("buffer length = {}", self.body.len());
        metric.format_influx(&mut self.body).map_err(|_| ())?;
        debug!("new buffer length = {}", self.body.len());
        Ok(())
    }

    pub(super) fn send_required(&self) -> bool {
        let free = self.body.capacity() - self.body.len();
        free < BUFFER_FREE_SPACE_THRESHOLD
    }

    pub(super) async fn tx<T: embedded_nal_async::TcpConnect, D: embedded_nal_async::Dns>(
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

        let mut request = match embassy_time::with_timeout(
            Duration::from_secs(3),
            http_client.request(Method::POST, TELEGRAF_URL),
        )
        .await
        {
            Ok(Ok(request)) => request
                .basic_auth(TELEGRAF_USERNAME, TELEGRAF_PASSWORD)
                .content_type(ContentType::TextPlain)
                .body(self.body.as_bytes()),
            Ok(Err(e)) => {
                warn!("Metrics submission failed: {}", e);
                TELEMETRY_TX_FAIL_NETWORK.add(1, Ordering::Relaxed);
                return;
            }
            Err(_) => {
                warn!("Metrics submission failed: timeout");
                TELEMETRY_TX_FAIL_NETWORK.add(1, Ordering::Relaxed);
                return;
            }
        };

        match embassy_time::with_timeout(Duration::from_secs(3), request.send(rx_buffer)).await {
            Ok(Ok(response)) => {
                if response.status == StatusCode(204) {
                    debug!("Metrics submission success: status={}", response.status);
                } else {
                    warn!("Metrics submission failed: status={}", response.status);
                    TELEMETRY_TX_FAIL_NETWORK.add(1, Ordering::Relaxed);

                    if response.status == StatusCode(400) {
                        warn!("Telegraf reports bad request, also clearing the buffer as this is probably a line format serialization issue");
                        self.body.clear();
                    }

                    return;
                }
            }
            Ok(Err(e)) => {
                warn!("Metrics submission failed: {}", e);
                TELEMETRY_TX_FAIL_NETWORK.add(1, Ordering::Relaxed);
                return;
            }
            Err(_) => {
                warn!("Metrics submission failed: timeout");
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
