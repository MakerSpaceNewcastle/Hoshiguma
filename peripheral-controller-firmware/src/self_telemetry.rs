//! Produces data points for the peripheral controller itself.
//!
//! Sent on boot:
//! - firmware Git revision
//! - boot reason
//!
//! Sent at a 1 minute interval:
//! - uptime
//! - number of data point template errors
//! - number of data points discarded due to buffer capacity

use crate::telemetry::queue_telemetry_data_point;
use core::sync::atomic::Ordering;
use embassy_time::{Instant, Timer};
use hoshiguma_core::telemetry::{StaticTelemetryDataPoint, TelemetryStrValue, TelemetryValue};
use portable_atomic::AtomicUsize;

pub(crate) static DATA_POINT_TEMPLATE_ERRORS: AtomicUsize = AtomicUsize::new(0);
pub(crate) static DATA_POINTS_DISCARDED: AtomicUsize = AtomicUsize::new(0);

#[embassy_executor::task]
pub(crate) async fn task() {
    // Send data points that only change on boot
    {
        queue_telemetry_data_point(StaticTelemetryDataPoint {
            measurement: "peripheral_controller_git_revision",
            field: "value",
            value: TelemetryValue::DynamicString(git_version::git_version!().try_into().unwrap()),
            timestamp: None,
        });

        queue_telemetry_data_point(StaticTelemetryDataPoint {
            measurement: "peripheral_controller_boot_reason",
            field: "value",
            value: TelemetryValue::StaticString(crate::boot_reason().telemetry_str()),
            timestamp: None,
        });
    }

    loop {
        queue_telemetry_data_point(StaticTelemetryDataPoint {
            measurement: "peripheral_controller_uptime",
            field: "value",
            value: TelemetryValue::U64(Instant::now().as_millis()),
            timestamp: None,
        });

        queue_telemetry_data_point(StaticTelemetryDataPoint {
            measurement: "peripheral_controller_data_point_template_errors",
            field: "value",
            value: TelemetryValue::Usize(DATA_POINT_TEMPLATE_ERRORS.load(Ordering::Relaxed)),
            timestamp: None,
        });

        queue_telemetry_data_point(StaticTelemetryDataPoint {
            measurement: "peripheral_controller_data_points_discarded",
            field: "value",
            value: TelemetryValue::Usize(DATA_POINTS_DISCARDED.load(Ordering::Relaxed)),
            timestamp: None,
        });

        // Send data points (approximately) every minute
        Timer::after_secs(60).await;
    }
}
