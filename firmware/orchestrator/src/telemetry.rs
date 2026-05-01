use crate::{
    self_telemetry::{DATA_POINTS_DISCARDED_FORMAT_FAIL, DATA_POINTS_DISCARDED_QUEUE_FULL},
    telemetry_bridge_comm::wait_for_telemetry_bridge_ready,
};
use core::sync::atomic::Ordering;
use defmt::{debug, info, warn};
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use hoshiguma_api::{
    CONTROL_PORT, TELEMETRY_BRIDGE_IP_ADDRESS,
    telemetry_bridge::{
        FormattedTelemetryDataPoint, Request, Response, ResponseData, TELEMETRY_DATA_POINT_MAX_LEN,
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

    // TODO
    // let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut data_point_being_sent: Option<FormattedTelemetryDataPoint> = None;

    'connection: loop {
        // Report telemetry inoperative
        // TODO
        // status_tx
        //     .publish((MonitorKind::TelemetryInop, Severity::Information))
        //     .await;

        // Wait for telemetry bridge to come online
        wait_for_telemetry_bridge_ready(stack).await;

        // Report telemetry ready
        // TODO
        // status_tx
        //     .publish((MonitorKind::TelemetryInop, Severity::Normal))
        //     .await;

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
            match send_request::<_, Response>(
                stack,
                TELEMETRY_BRIDGE_IP_ADDRESS,
                CONTROL_PORT,
                &Request::SendTelemetryDataPoint(data_point),
            )
            .await
            {
                Ok(response) => match response.0 {
                    Ok(ResponseData::TelementryDataPointAck) => {
                        debug!("Data point ack");
                        data_point_being_sent.take();
                    }
                    response => {
                        warn!("Unexpected response: {}", response);
                    }
                },
                Err(e) => {
                    warn!("Failed to send request: {}", e);
                    continue 'connection;
                }
            }
        }
    }
}
