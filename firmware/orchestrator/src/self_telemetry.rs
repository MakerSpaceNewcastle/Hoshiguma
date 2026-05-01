//! Produces data points for the peripheral controller itself.
//!
//! Sent on boot:
//! - firmware Git revision
//! - boot reason
//!
//! Sent at a 1 minute interval:
//! - uptime
//! - wall time
//! - number of data points discarded due to formatting failures
//! - number of data points discarded due to buffer capacity

use crate::telemetry::queue_telemetry_data_point;
use core::sync::atomic::Ordering;
use embassy_time::{Instant, Timer};
use hoshiguma_api::{GitRevisionString, TelemetryString};
use hoshiguma_common::telemetry::{format_influx_line, format_influx_line_str};
use portable_atomic::AtomicUsize;

pub(crate) static DATA_POINTS_DISCARDED_FORMAT_FAIL: AtomicUsize = AtomicUsize::new(0);
pub(crate) static DATA_POINTS_DISCARDED_QUEUE_FULL: AtomicUsize = AtomicUsize::new(0);
pub(crate) static DATA_POINTS_DISCARDED_TX_FAIL: AtomicUsize = AtomicUsize::new(0);

#[embassy_executor::task]
pub(crate) async fn task() {
    // Send data points that only change on boot
    {
        let git_revision: GitRevisionString = git_version::git_version!().try_into().unwrap();
        queue_telemetry_data_point(format_influx_line_str(
            "orchestrator_git_revision",
            "value",
            git_revision,
            None,
        ));

        queue_telemetry_data_point(format_influx_line_str(
            "orchestrator_boot_reason",
            "value",
            crate::boot_reason().telemetry_str(),
            None,
        ));
    }

    loop {
        queue_telemetry_data_point(format_influx_line(
            "orchestrator_uptime",
            "value",
            Instant::now().as_millis(),
            None,
        ));

        // TODO
        queue_telemetry_data_point(format_influx_line(
            "orchestrator_wall_time",
            "value",
            0, //crate::wall_time::now().unwrap_or_default().timestamp(),
            None,
        ));

        queue_telemetry_data_point(format_influx_line(
            "orchestrator_data_points_discarded,reason=format_error",
            "value",
            DATA_POINTS_DISCARDED_FORMAT_FAIL.load(Ordering::Relaxed),
            None,
        ));

        queue_telemetry_data_point(format_influx_line(
            "orchestrator_data_points_discarded,reason=queue_full",
            "value",
            DATA_POINTS_DISCARDED_QUEUE_FULL.load(Ordering::Relaxed),
            None,
        ));

        queue_telemetry_data_point(format_influx_line(
            "orchestrator_data_points_discarded,reason=tx_fail",
            "value",
            DATA_POINTS_DISCARDED_TX_FAIL.load(Ordering::Relaxed),
            None,
        ));

        // Send data points (approximately) every minute
        Timer::after_secs(60).await;
    }
}
