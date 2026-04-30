use crate::self_telemetry::{DATA_POINTS_DISCARDED_BUFFER, DATA_POINTS_DISCARDED_FORMAT};
use core::sync::atomic::Ordering;
use defmt::warn;
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use hoshiguma_api::telemetry_bridge::{FormattedTelemetryDataPoint, TELEMETRY_DATA_POINT_MAX_LEN};
use hoshiguma_common::telemetry::FormatInfluxResult;

static TELEMETRY_TX: Channel<CriticalSectionRawMutex, FormattedTelemetryDataPoint, 64> =
    Channel::new();

pub(crate) fn queue_telemetry_data_point(
    data_point: FormatInfluxResult<TELEMETRY_DATA_POINT_MAX_LEN>,
) {
    if let Ok(data_point) = data_point {
        if let Err(e) = TELEMETRY_TX.try_send(FormattedTelemetryDataPoint(data_point)) {
            warn!("Data point discarded: {}", e);
            DATA_POINTS_DISCARDED_BUFFER.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        warn!("Data point discarded: failed to format data point");
        DATA_POINTS_DISCARDED_FORMAT.fetch_add(1, Ordering::Relaxed);
    }
}

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("telemetry").await;

    let telem_rx = TELEMETRY_TX.receiver();

    // let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    'connection: loop {
        //         // Report telemetry inoperative
        //         status_tx
        //             .publish((MonitorKind::TelemetryInop, Severity::Information))
        //             .await;

        // Wait for telemetry module to come online
        wait_for_telemetry_module_ready(stack).await;

        //         // Report telemetry ready
        //         status_tx
        //             .publish((MonitorKind::TelemetryInop, Severity::Normal))
        //             .await;

        // Receive data points from queue
        loop {
            let data_point = TELEMETRY_TX.receive().await;

            // TODO
        }
    }
}

async fn wait_for_telemetry_module_ready(stack: Stack<'static>) {
    loop {
        match is_telemetry_module_ready(stack).await {
            Ok(true) => return,
            Ok(false) => (),
            Err(_) => (),
        }

        Timer::after_secs(1).await;
    }
}

async fn is_telemetry_module_ready(stack: Stack<'static>) -> Result<bool, ()> {
    todo!()
}
