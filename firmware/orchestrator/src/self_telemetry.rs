//! Produces data points for the peripheral controller itself.
//!
//! Sent on boot:
//! - firmware Git revision
//! - boot reason
//!
//! Sent at a 1 minute interval:
//! - uptime
//! - number of data points discarded due to formatting failures
//! - number of data points discarded due to buffer capacity

use crate::telemetry::queue_telemetry_data_point;
use core::sync::atomic::Ordering;
use embassy_time::{Instant, Timer};
use hoshiguma_api::{GitRevisionString, TelemetryString};
use hoshiguma_common::telemetry::{format_influx_line, format_influx_line_str};
use portable_atomic::AtomicUsize;

pub(crate) static DATA_POINTS_DISCARDED_FORMAT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static DATA_POINTS_DISCARDED_BUFFER: AtomicUsize = AtomicUsize::new(0);

#[embassy_executor::task]
pub(crate) async fn task() {
    // Send data points that only change on boot
    {
        let git_revision: GitRevisionString = git_version::git_version!().try_into().unwrap();
        queue_telemetry_data_point(format_influx_line_str(
            "peripheral_controller_git_revision",
            "value",
            git_revision,
            None,
        ));

        queue_telemetry_data_point(format_influx_line_str(
            "peripheral_controller_boot_reason",
            "value",
            crate::boot_reason().telemetry_str(),
            None,
        ));
    }

    loop {
        queue_telemetry_data_point(format_influx_line(
            "peripheral_controller_uptime",
            "value",
            Instant::now().as_millis(),
            None,
        ));

        queue_telemetry_data_point(format_influx_line(
            "peripheral_controller_data_points_discarded,reason=format_error",
            "value",
            DATA_POINTS_DISCARDED_FORMAT.load(Ordering::Relaxed),
            None,
        ));

        queue_telemetry_data_point(format_influx_line(
            "peripheral_controller_data_points_discarded,reason=buffer_capacity",
            "value",
            DATA_POINTS_DISCARDED_BUFFER.load(Ordering::Relaxed),
            None,
        ));

        // Send data points (approximately) every minute
        Timer::after_secs(60).await;
    }
}
