use crate::{
    logic::interlock::update_monitor_severity,
    self_telemetry::{DATA_POINTS_DISCARDED_FORMAT_FAIL, DATA_POINTS_DISCARDED_QUEUE_FULL},
    telemetry_bridge_comm::wait_for_telemetry_bridge_ready,
};
use core::sync::atomic::Ordering;
use defmt::{debug, info, warn};
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use hoshiguma_api::{
    API_PORT, Monitor, Severity, TELEMETRY_BRIDGE_IP_ADDRESS,
    telemetry_bridge::{
        FormattedTelemetryDataPoint, TELEMETRY_DATA_POINT_MAX_LEN, request, response,
    },
};
use hoshiguma_common::{network::send_request, telemetry::FormatInfluxResult};

static TELEMETRY_TX: Channel<CriticalSectionRawMutex, FormattedTelemetryDataPoint, 64> =
    Channel::new();

pub(crate) fn queue_telemetry_data_point(
    data_point: FormatInfluxResult<TELEMETRY_DATA_POINT_MAX_LEN>,
) {
    if let Ok(data_point) = data_point {
        if let Err(e) = TELEMETRY_TX.try_send(FormattedTelemetryDataPoint(data_point)) {
            warn!("Data point discarded: {}", e);
            DATA_POINTS_DISCARDED_QUEUE_FULL.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        warn!("Data point discarded: failed to format data point");
        DATA_POINTS_DISCARDED_FORMAT_FAIL.fetch_add(1, Ordering::Relaxed);
    }
}

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("telemetry").await;

    let telem_rx = TELEMETRY_TX.receiver();

    let mut data_point_being_sent: Option<FormattedTelemetryDataPoint> = None;

    'connection: loop {
        // Report telemetry inoperative
        update_monitor_severity(Monitor::TelemetryBridgeCommunication, Severity::Information).await;

        // Wait for telemetry bridge to come online
        wait_for_telemetry_bridge_ready(stack).await;

        // Report telemetry ready
        update_monitor_severity(Monitor::TelemetryBridgeCommunication, Severity::Normal).await;

        // Receive data points from queue
        loop {
            let data_point: FormattedTelemetryDataPoint = match data_point_being_sent {
                Some(ref point) => {
                    // Small delay because things don't fail for no reason
                    Timer::after_millis(100).await;

                    debug!("Retrying failed data point");
                    point.clone()
                }
                None => telem_rx.receive().await,
            };

            info!("Sending data point: {}", data_point.0);
            match send_request::<_, response::TelemetryDataPointAck>(
                stack,
                TELEMETRY_BRIDGE_IP_ADDRESS,
                API_PORT,
                5,
                &request::SendTelemetryDataPoint(data_point),
            )
            .await
            {
                Ok(_) => {
                    debug!("Data point ack");
                    data_point_being_sent.take();
                }
                Err(e) => {
                    warn!("Failed to send request: {}", e);
                    continue 'connection;
                }
            }
        }
    }
}
