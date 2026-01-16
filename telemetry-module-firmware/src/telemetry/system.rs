use crate::{
    metric::{Metric, MetricKind, TimeMetrics},
    network::{
        LINK_STATE,
        telemetry_tx::{
            METRIC_TX, TELEMETRY_TX_BUFFER_SUBMISSIONS, TELEMETRY_TX_FAIL_BUFFER,
            TELEMETRY_TX_FAIL_NETWORK, TELEMETRY_TX_SUCCESS,
        },
    },
    telemetry::machine::{TELEMETRY_RX_FAIL, TELEMETRY_RX_SUCCESS},
};
use core::sync::atomic::Ordering;
use embassy_time::Timer;

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("system telemetry").await;

    let metric_tx = METRIC_TX.publisher().unwrap();

    loop {
        // Send system metrics every minute
        Timer::after_secs(60).await;

        let now = crate::network::time::wall_time();

        metric_tx
            .publish(Metric::new(
                now,
                MetricKind::TelemetryModuleSystemInformation(crate::system_information()),
            ))
            .await;

        metric_tx
            .publish(Metric::new(
                now,
                MetricKind::TelemetryModuleNetworkState(LINK_STATE.lock(|v| v.borrow().clone())),
            ))
            .await;

        metric_tx
            .publish(Metric::new(
                now,
                MetricKind::TelemetryModuleTime(TimeMetrics {
                    unix_epoch_nanoseconds: crate::network::time::wall_time()
                        .unwrap_or_default()
                        .as_nanos() as u64,
                    sync_age_nanoseconds: crate::network::time::time_sync_age().as_millis() as u64,
                }),
            ))
            .await;

        metric_tx
            .publish(Metric::new(
                now,
                MetricKind::TelemetryModuleStatistics(crate::metric::Statistics {
                    events_received: TELEMETRY_RX_SUCCESS.load(Ordering::Relaxed),
                    receive_failures: TELEMETRY_RX_FAIL.load(Ordering::Relaxed),
                    metrics_added_to_transmit_buffer: TELEMETRY_TX_BUFFER_SUBMISSIONS
                        .load(Ordering::Relaxed),
                    messages_transmitted: TELEMETRY_TX_SUCCESS.load(Ordering::Relaxed),
                    tramsit_buffer_failures: TELEMETRY_TX_FAIL_BUFFER.load(Ordering::Relaxed),
                    transmit_network_failures: TELEMETRY_TX_FAIL_NETWORK.load(Ordering::Relaxed),
                }),
            ))
            .await;
    }
}
