use crate::self_telemetry::{TELEGRAF_SUBMIT_FAIL, TELEGRAF_SUBMIT_SUCCESS};
use core::{fmt::Write, sync::atomic::Ordering};
use defmt::{Format, debug, info, warn};
use embassy_time::{Duration, WithTimeout};
use heapless::String;
use reqwless::{
    client::HttpClient,
    headers::ContentType,
    request::{Method, RequestBuilder},
    response::StatusCode,
};

#[derive(Format, Default)]
pub(crate) struct TelegrafBuffer {
    body: String<12288>,
}

impl TelegrafBuffer {
    pub(crate) fn push<const LEN: usize>(&mut self, line: String<LEN>) -> Result<(), ()> {
        info!("New line: {}", line);
        debug!("buffer length = {}", self.body.len());
        self.body.write_str(&line).map_err(|_| ())?;
        self.body.write_str("\n").map_err(|_| ())?;
        debug!("new buffer length = {}", self.body.len());
        Ok(())
    }

    pub(crate) fn send_required(&self) -> bool {
        let free = self.body.capacity() - self.body.len();
        free < 2048
    }

    pub(crate) async fn tx<T: embedded_nal_async::TcpConnect, D: embedded_nal_async::Dns>(
        &mut self,
        http_client: &mut HttpClient<'_, T, D>,
        rx_buffer: &mut [u8],
    ) -> Result<(), ()> {
        if self.body.is_empty() {
            // Buffer is empty, nothing to do
            debug!("No data to submit");
            return Ok(());
        }

        const TELEGRAF_URL: &str = env!("TELEGRAF_URL");
        const TELEGRAF_USERNAME: &str = env!("TELEGRAF_USERNAME");
        const TELEGRAF_PASSWORD: &str = env!("TELEGRAF_PASSWORD");

        info!("Submitting metrics to {}", &TELEGRAF_URL);

        let mut request = match http_client
            .request(Method::POST, TELEGRAF_URL)
            .with_timeout(Duration::from_secs(2))
            .await
        {
            Ok(Ok(request)) => request
                .basic_auth(TELEGRAF_USERNAME, TELEGRAF_PASSWORD)
                .content_type(ContentType::TextPlain)
                .body(self.body.as_bytes()),
            Ok(Err(e)) => {
                warn!("Metrics submission failed: {}", e);
                TELEGRAF_SUBMIT_FAIL.add(1, Ordering::Relaxed);
                return Err(());
            }
            Err(_) => {
                warn!("Metrics submission failed: timeout");
                TELEGRAF_SUBMIT_FAIL.add(1, Ordering::Relaxed);
                return Err(());
            }
        };

        match request
            .send(rx_buffer)
            .with_timeout(Duration::from_secs(2))
            .await
        {
            Ok(Ok(response)) => {
                if response.status == StatusCode(204) {
                    debug!("Metrics submission success: status={}", response.status);
                } else {
                    warn!("Metrics submission failed: status={}", response.status);
                    TELEGRAF_SUBMIT_FAIL.add(1, Ordering::Relaxed);

                    if response.status == StatusCode(400) {
                        warn!(
                            "Telegraf reports bad request, also clearing the buffer as this is probably a line format serialization issue"
                        );
                        self.body.clear();
                    }

                    return Err(());
                }
            }
            Ok(Err(e)) => {
                warn!("Metrics submission failed: {}", e);
                TELEGRAF_SUBMIT_FAIL.add(1, Ordering::Relaxed);
                return Err(());
            }
            Err(_) => {
                warn!("Metrics submission failed: timeout");
                TELEGRAF_SUBMIT_FAIL.add(1, Ordering::Relaxed);
                return Err(());
            }
        };

        // Clear the buffer once transmitted
        self.body.clear();

        debug!("Metric submission successful");
        TELEGRAF_SUBMIT_SUCCESS.add(1, Ordering::Relaxed);
        Ok(())
    }
}
